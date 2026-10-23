//! `Tooltip`: a small popup with a short text after the pointer rests on a node.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, Pressed, UiGlobalTransform};
use bevy::window::PrimaryWindow;

use crate::float::Float;
use crate::theme::Theme;

/// How long the pointer must rest on a [`Tooltip`] node before the popup shows, in seconds.
pub const TOOLTIP_DELAY_SECS: f32 = 0.5;

/// The distance from the window edge the popup keeps, in logical pixels.
const MARGIN: f32 = 6.0;
/// The gap between the node and its popup, in logical pixels.
const GAP: f32 = 6.0;

/// Registers the systems and observers of [`Tooltip`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(despawn_popup_with_owner).add_systems(
            Update,
            (
                update_tooltips,
                place_popups,
                restyle_popups.run_if(resource_changed::<Theme>),
            )
                .chain(),
        );
    }
}

/// A short text shown in a popup below the node after the pointer has rested on it for
/// [`TOOLTIP_DELAY_SECS`].
///
/// The popup is a [`Float`] child of the node, placed in the viewport below the node and kept
/// inside the window, and goes above it when it would run off the bottom. It hides when the
/// pointer leaves the node or presses it (and does not come back until the pointer has left
/// and returned), and it never takes the pointer. An
/// [`IconButton`](crate::icon_button::IconButton) has one with its name, and so has the button
/// of an underlined [`Toolbar`](crate::toolbar::Toolbar), whose labels are hidden.
///
/// Insert a `Tooltip` again to change the text.
#[derive(Component, Clone, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[require(Hovered, TooltipClock)]
pub struct Tooltip {
    /// The text of the popup.
    pub text: String,
}

impl Tooltip {
    /// A tooltip showing `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

/// How long the pointer has rested on a [`Tooltip`] node.
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
struct TooltipClock {
    /// Seconds the pointer has been on the node without being pressed.
    elapsed: f32,
    /// Set by a press: no popup until the pointer has left.
    suppressed: bool,
}

/// The popup of a [`Tooltip`] node that is showing.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct TooltipPopup {
    /// The node the popup belongs to.
    pub owner: Entity,
}

/// The text of a popup.
#[derive(Component, Clone, Copy, Debug)]
struct TooltipText;

/// The popup of a node, set while it shows.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
struct TooltipRef(Entity);

fn popup_node(theme: &Theme) -> Node {
    Node {
        position_type: PositionType::Absolute,
        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
        border: UiRect::all(theme.border_thickness),
        ..default()
    }
}

fn text_font(theme: &Theme) -> TextFont {
    TextFont {
        font: theme.body_font.clone().into(),
        font_size: Theme::font_size(theme.font_size_small),
        ..default()
    }
}

fn update_tooltips(
    time: Res<Time>,
    mut nodes: Query<(
        Entity,
        &Tooltip,
        &Hovered,
        Has<Pressed>,
        &mut TooltipClock,
        Option<&TooltipRef>,
    )>,
    mut popup_texts: Query<&mut Text, With<TooltipText>>,
    popups: Query<&Children, With<TooltipPopup>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for (entity, tooltip, hovered, pressed, mut clock, popup) in &mut nodes {
        if !hovered.get() {
            *clock = TooltipClock::default();
        } else if pressed {
            clock.suppressed = true;
            clock.elapsed = 0.0;
        } else if !clock.suppressed {
            clock.elapsed += time.delta_secs();
        }
        let wanted = hovered.get() && !clock.suppressed && clock.elapsed >= TOOLTIP_DELAY_SECS;
        match (wanted, popup) {
            (true, None) => {
                let popup = spawn_popup(&mut commands, &theme, entity, &tooltip.text);
                commands.entity(entity).insert(TooltipRef(popup));
            }
            (true, Some(TooltipRef(popup))) => {
                // Follow a changed text.
                if let Ok(children) = popups.get(*popup) {
                    for child in children {
                        if let Ok(mut text) = popup_texts.get_mut(*child)
                            && text.0 != tooltip.text
                        {
                            text.0.clone_from(&tooltip.text);
                        }
                    }
                }
            }
            (false, Some(TooltipRef(popup))) => {
                commands.entity(*popup).despawn();
                commands.entity(entity).remove::<TooltipRef>();
            }
            (false, None) => {}
        }
    }
}

fn spawn_popup(commands: &mut Commands, theme: &Theme, owner: Entity, text: &str) -> Entity {
    let popup = commands
        .spawn((
            Name::new("Tooltip popup"),
            TooltipPopup { owner },
            Float,
            popup_node(theme),
            BackgroundColor(theme.colors.card_surface),
            BorderColor::all(theme.colors.border),
            // Hidden until `place_popups` has the laid-out size to place it with.
            Visibility::Hidden,
            Pickable::IGNORE,
            ChildOf(owner),
        ))
        .id();
    commands.spawn((
        TooltipText,
        Text::new(text),
        text_font(theme),
        TextColor(theme.colors.text_primary),
        Pickable::IGNORE,
        ChildOf(popup),
    ));
    popup
}

fn despawn_popup_with_owner(
    t: On<Remove<Tooltip>>,
    refs: Query<&TooltipRef>,
    mut commands: Commands,
) {
    // The popup is a child of the node, so it goes with a despawned node by itself.
    let node = t.event().event_target();
    if let Ok(TooltipRef(popup)) = refs.get(node) {
        commands.entity(*popup).try_despawn();
    }
    // `update_tooltips` only visits nodes with a `Tooltip`, so a ref left behind would stay stale
    // and stop the popup showing once a `Tooltip` is inserted again. (A despawning node ignores
    // these.)
    commands
        .entity(node)
        .try_remove::<TooltipRef>()
        .try_insert(TooltipClock::default());
}

/// Puts each popup centred under its node, inside the window, or above the node when it would
/// run off the bottom, and shows it. Needs the popup's laid-out size, so it shows one frame
/// after it is spawned.
fn place_popups(
    window: Query<&Window, With<PrimaryWindow>>,
    mut popups: Query<(&TooltipPopup, &ComputedNode, &mut Node, &mut Visibility)>,
    owners: Query<(&ComputedNode, &UiGlobalTransform), Without<TooltipPopup>>,
) {
    let window = window.iter().next();
    for (popup, popup_computed, mut node, mut visibility) in &mut popups {
        let Ok((owner_computed, owner_transform)) = owners.get(popup.owner) else {
            continue;
        };
        let scale = owner_computed.inverse_scale_factor();
        let owner_size = owner_computed.size() * scale;
        let popup_size = popup_computed.size() * popup_computed.inverse_scale_factor();
        if popup_size == Vec2::ZERO {
            continue;
        }
        // `UiGlobalTransform` holds the centre of the node.
        let center = owner_transform.translation * scale;
        let mut left = center.x - popup_size.x / 2.0;
        let below = center.y + owner_size.y / 2.0 + GAP;
        let mut top = below;
        if let Some(window) = window {
            left = left.min(window.width() - MARGIN - popup_size.x).max(MARGIN);
            if below + popup_size.y + MARGIN > window.height() {
                top = center.y - owner_size.y / 2.0 - GAP - popup_size.y;
            }
        }
        node.left = Val::Px(left);
        node.top = Val::Px(top);
        if *visibility != Visibility::Inherited {
            *visibility = Visibility::Inherited;
        }
    }
}

/// Restyles the popups that are showing after a theme change.
fn restyle_popups(
    theme: Res<Theme>,
    mut popups: Query<
        (&mut Node, &mut BackgroundColor, &mut BorderColor, &Children),
        With<TooltipPopup>,
    >,
    mut texts: Query<(&mut TextFont, &mut TextColor), With<TooltipText>>,
) {
    for (mut node, mut background, mut border, children) in &mut popups {
        node.border = UiRect::all(theme.border_thickness);
        background.0 = theme.colors.card_surface;
        *border = BorderColor::all(theme.colors.border);
        for child in children {
            if let Ok((mut font, mut color)) = texts.get_mut(*child) {
                *font = text_font(&theme);
                color.0 = theme.colors.text_primary;
            }
        }
    }
}
