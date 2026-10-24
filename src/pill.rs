//! `Pill`: a small uppercase status tag with a tone, such as "RUNNING".

use accesskit::{Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::text::{LineBreak, TextLayout};
use itertools::Either;

use crate::theme::Theme;

/// The text size of a pill, in pixels.
const PILL_FONT_SIZE: f32 = 11.0;

/// Registers the systems and observers of [`Pill`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct PillPlugin;

impl Plugin for PillPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_pills)
                .run_if(resource_changed::<Theme>),
        );
    }
}

/// What a [`Pill`] says about its subject. The application picks it; the
/// library does not know what a status means.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum PillTone {
    /// Nothing in particular.
    #[default]
    Neutral,
    /// Informational, such as running.
    Info,
    /// Waiting.
    Warning,
    /// Something is wrong, such as no path.
    Problem,
}

/// A status tag: short uppercase text on a rounded, tinted background.
///
/// The text is shown in uppercase, semibold, and never wraps. Insert a new
/// `Pill` on a live entity to change its tone or text. For a screen reader it is a status named
/// by the text as given. Empty text keeps the empty tinted pill (its padding) and an empty label;
/// it is not hidden.
///
/// The component is immutable: build or clone a value, edit it outside the world and insert it
/// again to update a live widget. It cannot be queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use rin_bevy_ui::pill::Pill;
/// fn edit(mut widgets: Query<&mut Pill>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert=Self::on_insert)]
#[require(
    Name::new("Pill"),
    Node {
        padding: UiRect::axes(Val::Px(8.0), Val::Px(5.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        flex_shrink: 0.0,
        ..default()
    },
    BackgroundColor,
    TextLayout {
        linebreak: LineBreak::NoWrap,
        ..default()
    },
    Pickable {
        should_block_lower: false,
        ..default()
    },
)]
pub struct Pill {
    /// The text, as given; it is shown in uppercase.
    pub text: String,
    /// How the pill is coloured.
    pub tone: PillTone,
}

impl Pill {
    /// A neutral pill showing `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: PillTone::Neutral,
        }
    }

    /// The same pill with `tone`.
    #[must_use]
    pub const fn with_tone(mut self, tone: PillTone) -> Self {
        self.tone = tone;
        self
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let Some(this) = world.entity(context.entity).get::<Self>() else {
            return;
        };
        let text = this.text.to_uppercase();
        let mut node = AccessNode::new(Role::Status);
        node.set_label(this.text.as_str());
        world
            .commands()
            .entity(context.entity)
            .insert((Text(text), AccessibilityNode::from(node)));
        world
            .commands()
            .run_system_cached_with(theme_pills, Some(context.entity));
    }
}

fn theme_pills(
    pill: In<Option<Entity>>,
    theme: Res<Theme>,
    mut pills: Query<(&Pill, &mut BackgroundColor, &mut TextColor, &mut TextFont)>,
) {
    let iter = if let Some(pill) = pill.0 {
        Either::Left(pills.get_mut(pill).into_iter())
    } else {
        Either::Right(pills.iter_mut())
    };

    for (pill, mut background, mut text_color, mut text_font) in iter {
        let colors = &theme.colors;
        let (bg, text) = match pill.tone {
            PillTone::Neutral => (colors.pill_background, colors.pill_text),
            PillTone::Info => (colors.info_background, colors.info_text),
            PillTone::Warning => (colors.wait_background, colors.wait_text),
            PillTone::Problem => (colors.warning_background, colors.warning_text),
        };
        background.0 = bg;
        text_color.0 = text;
        *text_font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(PILL_FONT_SIZE),
            weight: FontWeight::SEMIBOLD,
            ..default()
        };
    }
}
