//! `Badge`: a count on the top-right corner of a button.

use bevy::a11y::AccessibilityNode;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::text::{DEFAULT_REM_SIZE_PX, LineBreak, TextLayout};

use crate::select::Item;
use crate::theme::Theme;
use crate::toolbar::{Tool, ToolbarButton};

/// The height, and the minimum width, of a badge, in logical pixels at the default rem size.
///
/// It is laid out in rem, so it grows with the text: in pixels a larger rem size would clip
/// the count.
const BADGE_SIZE_PX: f32 = 16.0;
/// The padding to the left and right of the text of a badge, in logical pixels.
const BADGE_PADDING_PX: f32 = 4.0;
/// How far the badge sticks out of the top and right edge of its button, in logical pixels.
const BADGE_OFFSET_PX: f32 = -2.0;
/// The text size of a badge, in logical pixels.
const BADGE_FONT_SIZE_PX: f32 = 10.0;

/// Registers the systems of [`Badge`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct BadgePlugin;

impl Plugin for BadgePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (mirror_item_badges, sync_badges).chain());
    }
}

/// What a [`Badge`] says about its button. The application picks it; the library does not
/// know what a count means.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum BadgeTone {
    /// An ordinary count, such as the number of trains.
    #[default]
    Neutral,
    /// Something to look at, such as the number of open warnings.
    Warning,
}

/// A pill with a count or a short text on the top-right corner of a button.
///
/// Put it on an [`IconButton`](crate::icon_button::IconButton), or on the
/// [`Item<Tool>`](crate::toolbar::Tool) of a [`Toolbar`](crate::toolbar::Toolbar) to badge that
/// tool's button. The badge is 16 px high with a minimum width of 16 px, and does not block the
/// pointer from the button under it. It is not in the accessibility tree itself: its text is the
/// description of the button's accessibility node. The application sets and updates the text by inserting a
/// new `Badge`; removing the component, or an empty text, shows nothing (no overlay is drawn and
/// the host's description is cleared), and inserting a nonempty one again brings it back. A
/// badge mirrored from an `Item<Tool>` is updated by inserting a new `Badge` on the item.
///
/// The component is immutable: build or clone a value, edit it outside the world and insert it
/// again to update a live widget. It cannot be queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::badge::Badge;
/// fn edit(mut widgets: Query<&mut Badge>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct Badge {
    /// The text of the badge, usually a count. Empty shows no badge.
    pub text: String,
    /// How the badge is coloured.
    pub tone: BadgeTone,
}

impl Badge {
    /// A neutral badge showing `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tone: BadgeTone::Neutral,
        }
    }

    /// A neutral badge showing `count`; a count of 0 shows "0", so the application removes the
    /// badge when it wants none.
    #[must_use]
    pub fn count(count: usize) -> Self {
        Self::new(count.to_string())
    }

    /// The same badge with `tone`.
    #[must_use]
    pub const fn with_tone(mut self, tone: BadgeTone) -> Self {
        self.tone = tone;
        self
    }
}

/// The badge node spawned under a button that has a [`Badge`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub struct BadgeRef(pub Entity);

/// The node that draws a [`Badge`].
#[derive(Component, Clone, Copy, Debug)]
struct BadgeLabel;

/// Gives the button of a tool the [`Badge`] of its item, and takes it off when the item has
/// none.
///
/// Looks only at what a change concerns: a button that is new or whose [`ToolbarButton`] was
/// inserted again (a replaced item), and the buttons of an item whose [`Badge`] changed or was
/// removed or that is new. A quiet frame reads no button.
fn mirror_item_badges(
    changed_items: Query<Entity, (With<Item<Tool>>, Or<(Changed<Badge>, Added<Item<Tool>>)>)>,
    new_buttons: Query<Entity, Changed<ToolbarButton>>,
    mut removed: RemovedComponents<Badge>,
    buttons: Query<(Entity, &ToolbarButton, Option<&Badge>)>,
    items: Query<Option<&Badge>, With<Item<Tool>>>,
    mut touched: Local<HashSet<Entity>>,
    mut commands: Commands,
) {
    touched.clear();
    touched.extend(changed_items.iter().chain(removed.read()));
    let new_buttons: Vec<Entity> = new_buttons.iter().collect();
    if touched.is_empty() && new_buttons.is_empty() {
        return;
    }
    let mut mirror = |entity: Entity, button: &ToolbarButton, current: Option<&Badge>| {
        let wanted = items.get(button.item()).ok().flatten();
        if wanted == current {
            return;
        }
        match wanted {
            Some(badge) => commands.entity(entity).insert(badge.clone()),
            None => commands.entity(entity).remove::<Badge>(),
        };
    };
    for entity in new_buttons {
        if let Ok((entity, button, current)) = buttons.get(entity) {
            mirror(entity, button, current);
        }
    }
    if touched.is_empty() {
        return;
    }
    // A button whose own badge was taken off is looked at again too.
    for entity in &*touched {
        if let Ok((entity, button, current)) = buttons.get(*entity) {
            mirror(entity, button, current);
        }
    }
    for (entity, button, current) in &buttons {
        if touched.contains(&button.item()) {
            mirror(entity, button, current);
        }
    }
}

fn badge_font(theme: &Theme) -> TextFont {
    TextFont {
        font: theme.body_font.clone().into(),
        font_size: Theme::font_size(BADGE_FONT_SIZE_PX),
        weight: FontWeight::SEMIBOLD,
        ..default()
    }
}

fn badge_node() -> Node {
    Node {
        position_type: PositionType::Absolute,
        top: Val::Px(BADGE_OFFSET_PX),
        right: Val::Px(BADGE_OFFSET_PX),
        height: Val::Rem(BADGE_SIZE_PX / DEFAULT_REM_SIZE_PX),
        min_width: Val::Rem(BADGE_SIZE_PX / DEFAULT_REM_SIZE_PX),
        padding: UiRect::horizontal(Val::Px(BADGE_PADDING_PX)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        border_radius: BorderRadius::all(Val::Rem(BADGE_SIZE_PX / 2.0 / DEFAULT_REM_SIZE_PX)),
        ..default()
    }
}

/// Spawns, updates and despawns the badge node of every button to match its [`Badge`] and the
/// theme.
///
/// Looks only at the hosts whose [`Badge`] changed or was removed, that are new, or whose
/// accessibility node changed (Bevy rewrites it, which would drop the description), and at all
/// of them when the [`Theme`] changes.
fn sync_badges(
    theme: Res<Theme>,
    mut removed: RemovedComponents<Badge>,
    mut touched: Local<HashSet<Entity>>,
    mut hosts: ParamSet<(
        Query<
            Entity,
            (
                With<Node>,
                Or<(With<Badge>, With<BadgeRef>)>,
                Or<(Changed<Badge>, Added<Node>, Changed<AccessibilityNode>)>,
            ),
        >,
        Query<
            (
                Entity,
                Option<&Badge>,
                Option<&BadgeRef>,
                Option<&mut AccessibilityNode>,
            ),
            (With<Node>, Or<(With<Badge>, With<BadgeRef>)>),
        >,
    )>,
    mut labels: Query<
        (
            &mut Text,
            &mut TextColor,
            &mut BackgroundColor,
            &mut TextFont,
        ),
        With<BadgeLabel>,
    >,
    mut commands: Commands,
) {
    touched.clear();
    touched.extend(hosts.p0().iter().chain(removed.read()));
    let all = theme.is_changed();
    if !all && touched.is_empty() {
        return;
    }
    let colors = theme.colors();
    let wanted: Vec<Entity> = if all {
        hosts.p1().iter().map(|host| host.0).collect()
    } else {
        touched.iter().copied().collect()
    };
    let mut hosts = hosts.p1();
    for host in wanted {
        let Ok((host, badge, badge_ref, mut access)) = hosts.get_mut(host) else {
            continue;
        };
        let badge = badge.filter(|b| !b.text.is_empty());
        // The badge node stays out of the accessibility tree; its text describes the host.
        if let Some(access) = access.as_mut() {
            match badge {
                Some(badge) if access.description() != Some(badge.text.as_str()) => {
                    access.set_description(badge.text.as_str());
                }
                None if access.description().is_some() => access.clear_description(),
                _ => {}
            }
        }
        let Some(badge) = badge else {
            if let Some(BadgeRef(label)) = badge_ref {
                commands.entity(*label).despawn();
                commands.entity(host).remove::<BadgeRef>();
            }
            continue;
        };
        let (background, text_color) = match badge.tone {
            BadgeTone::Neutral => (colors.pill_background, colors.pill_text),
            BadgeTone::Warning => (colors.warning_text, colors.warning_background),
        };
        if let Some((mut text, mut color, mut fill, mut font)) =
            badge_ref.and_then(|r| labels.get_mut(r.0).ok())
        {
            if text.0 != badge.text {
                badge.text.clone_into(&mut text.0);
            }
            color.set_if_neq(TextColor(text_color));
            fill.set_if_neq(BackgroundColor(background));
            if theme.is_changed() {
                *font = badge_font(&theme);
            }
        } else if badge_ref.is_none() {
            let label = commands
                .spawn((
                    Name::new("Badge"),
                    BadgeLabel,
                    badge_node(),
                    BackgroundColor(background),
                    Text::new(badge.text.clone()),
                    TextColor(text_color),
                    badge_font(&theme),
                    TextLayout {
                        linebreak: LineBreak::NoWrap,
                        justify: Justify::Center,
                    },
                    Pickable::IGNORE,
                    ChildOf(host),
                ))
                .id();
            commands.entity(host).insert(BadgeRef(label));
        }
    }
}
