//! `LineBadge`: short text on a circle or pill in a color the application gives, such as the
//! badge of a train line.

use accesskit::{Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::text::{DEFAULT_REM_SIZE_PX, LineBreak, TextLayout};
use itertools::Either;

use crate::theme::{Theme, readable_ink};

/// The longest text that still makes a circle, in characters.
pub const LINE_BADGE_MAX_CIRCLE_CHARS: usize = 3;

/// The height of a regular badge, in logical pixels at the default rem size.
const REGULAR_HEIGHT_PX: f32 = 32.0;
/// The text size of a regular badge, in logical pixels at the default rem size.
const REGULAR_FONT_SIZE_PX: f32 = 14.0;
/// The height of a small badge, in logical pixels at the default rem size.
const SMALL_HEIGHT_PX: f32 = 24.0;
/// The text size of a small badge, in logical pixels at the default rem size.
const SMALL_FONT_SIZE_PX: f32 = 10.0;

/// Registers the systems of [`LineBadge`].
#[derive(Copy, Clone, Debug)]
pub struct LineBadgePlugin;

impl Plugin for LineBadgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_line_badges)
                .run_if(resource_changed::<Theme>),
        );
    }
}

/// How big a [`LineBadge`] is. Both sizes are in rem, so they follow [`RemSize`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum LineBadgeSize {
    /// For list rows: 2 rem high at the default rem size.
    #[default]
    Regular,
    /// For inline use: 1.5 rem high at the default rem size.
    Small,
}

impl LineBadgeSize {
    const fn height_px(self) -> f32 {
        match self {
            Self::Regular => REGULAR_HEIGHT_PX,
            Self::Small => SMALL_HEIGHT_PX,
        }
    }

    const fn font_size_px(self) -> f32 {
        match self {
            Self::Regular => REGULAR_FONT_SIZE_PX,
            Self::Small => SMALL_FONT_SIZE_PX,
        }
    }
}

/// The badge of a line: bold, centered text that never wraps, on a circle (up to three
/// characters) or a pill of the same height (more), filled with a color the application gives.
///
/// The text is white or near-black, whichever contrasts better with the fill, so the badge reads
/// in the light and the dark theme alike. A screen reader gets an image labeled with the text,
/// or with `label` when the application gives one. Immutable: insert a new value to change it.
/// It cannot be queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::line_badge::LineBadge;
/// fn edit(mut widgets: Query<&mut LineBadge>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert=Self::on_insert)]
#[require(
    Name::new("LineBadge"),
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        flex_shrink: 0.0,
        ..default()
    },
    BackgroundColor,
    TextLayout {
        linebreak: LineBreak::NoWrap,
        justify: Justify::Center,
    },
    Pickable {
        should_block_lower: false,
        ..default()
    },
)]
pub struct LineBadge {
    /// The text, up to five characters such as `L3`, `IC12` or `7`; shown as given.
    pub text: String,
    /// The fill color.
    pub color: Color,
    /// How big the badge is.
    pub size: LineBadgeSize,
    /// What a screen reader says instead of the text, such as "Line 3, Coal run".
    pub label: Option<String>,
}

impl LineBadge {
    /// A regular badge showing `text` on `color`.
    pub fn new(text: impl Into<String>, color: impl Into<Color>) -> Self {
        Self {
            text: text.into(),
            color: color.into(),
            size: LineBadgeSize::Regular,
            label: None,
        }
    }

    /// The same badge with `size`.
    #[must_use]
    pub const fn with_size(mut self, size: LineBadgeSize) -> Self {
        self.size = size;
        self
    }

    /// The same badge with the screen reader label `label`.
    #[must_use]
    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Whether the badge is a circle: its text has at most three characters. Longer text makes
    /// a pill.
    #[must_use]
    pub fn is_circle(&self) -> bool {
        self.text.chars().count() <= LINE_BADGE_MAX_CIRCLE_CHARS
    }

    /// The text color for a badge filled with `fill`: white or near-black, whichever has the
    /// higher contrast ratio with it.
    #[must_use]
    pub fn text_color_for(fill: Color) -> Color {
        readable_ink(fill)
    }

    /// What a screen reader says: `label`, or else the text.
    #[must_use]
    pub fn accessible_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.text)
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let Some(this) = world.entity(context.entity).get::<Self>() else {
            return;
        };
        let text = this.text.clone();
        let mut node = AccessNode::new(Role::Image);
        node.set_label(this.accessible_label());
        world
            .commands()
            .entity(context.entity)
            .insert((Text(text), AccessibilityNode::from(node)));
        world
            .commands()
            .run_system_cached_with(theme_line_badges, Some(context.entity));
    }
}

fn theme_line_badges(
    badge: In<Option<Entity>>,
    theme: Res<Theme>,
    mut badges: Query<(
        &LineBadge,
        &mut Node,
        &mut BackgroundColor,
        &mut TextColor,
        &mut TextFont,
    )>,
) {
    let iter = if let Some(badge) = badge.0 {
        Either::Left(badges.get_mut(badge).into_iter())
    } else {
        Either::Right(badges.iter_mut())
    };

    for (badge, mut node, mut background, mut text_color, mut text_font) in iter {
        let height = Val::Rem(badge.size.height_px() / DEFAULT_REM_SIZE_PX);
        let (width, min_width, padding) = if badge.is_circle() {
            (height, height, Val::ZERO)
        } else {
            (
                Val::Auto,
                height,
                Val::Rem(badge.size.height_px() * 0.4 / DEFAULT_REM_SIZE_PX),
            )
        };
        node.set_if_neq(Node {
            height,
            width,
            min_width,
            padding: UiRect::horizontal(padding),
            border_radius: BorderRadius::all(Val::Rem(
                badge.size.height_px() / 2.0 / DEFAULT_REM_SIZE_PX,
            )),
            ..node.clone()
        });
        background.set_if_neq(BackgroundColor(badge.color));
        text_color.set_if_neq(TextColor(LineBadge::text_color_for(badge.color)));
        text_font.set_if_neq(TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(badge.size.font_size_px()),
            weight: FontWeight::BOLD,
            ..default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::contrast_ratio;
    use bevy::color::palettes::tailwind::{AMBER_300, BLUE_700, RED_600, YELLOW_300};

    #[test]
    fn up_to_three_characters_make_a_circle_and_more_make_a_pill() {
        for (text, circle) in [
            ("", true),
            ("7", true),
            ("L3", true),
            ("IC1", true),
            ("IC12", false),
            ("ABC12", false),
        ] {
            assert_eq!(LineBadge::new(text, RED_600).is_circle(), circle, "{text}");
        }
    }

    #[test]
    fn the_text_is_white_on_a_dark_fill_and_near_black_on_a_light_one() {
        for dark in [BLUE_700, RED_600] {
            assert_eq!(LineBadge::text_color_for(dark.into()), Color::WHITE);
        }
        for light in [YELLOW_300, AMBER_300] {
            assert_ne!(LineBadge::text_color_for(light.into()), Color::WHITE);
        }
    }

    #[test]
    fn the_text_color_has_at_least_the_contrast_of_the_other_choice() {
        for fill in [
            Color::WHITE,
            Color::BLACK,
            RED_600.into(),
            YELLOW_300.into(),
        ] {
            let chosen = contrast_ratio(LineBadge::text_color_for(fill), fill);
            assert!(chosen >= contrast_ratio(Color::WHITE, fill));
            assert!(chosen >= 3.0, "{fill:?} has contrast {chosen}");
        }
    }

    #[test]
    fn the_label_is_the_text_unless_the_application_gives_one() {
        let badge = LineBadge::new("L3", RED_600);
        assert_eq!(badge.accessible_label(), "L3");
        assert_eq!(
            badge.with_label("Line 3, Coal run").accessible_label(),
            "Line 3, Coal run"
        );
    }
}
