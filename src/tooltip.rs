//! `Tooltip`: a small popup with a short text after the pointer rests on a node.

use bevy::picking::hover::Hovered;
use bevy::picking::pointer::PointerId;
use bevy::prelude::*;
use bevy::ui::{ComputedNode, Pressed, UiGlobalTransform};

use crate::float::{Float, UiViewport};
use crate::theme::Theme;
use crate::{DRAG_THRESHOLD, HoverLock, MouseEvent, MouseKind};

/// How long the pointer must rest on a [`Tooltip`] node before the popup shows, in seconds.
pub const TOOLTIP_DELAY_SECS: f32 = 0.5;

/// The distance from the window edge the popup keeps, in logical pixels.
const MARGIN: f32 = 6.0;
/// The gap between the node and its popup, in logical pixels.
const GAP: f32 = 6.0;

/// Marks an entity whose [`Tooltip`] a widget added, holding the text it gave it, so that the
/// widget replaces or removes only that one: a [`Tooltip`] with other text is the app's.
#[derive(Component, Debug, Clone)]
pub(crate) struct OwnedTooltip(pub String);

/// Registers the systems and observers of [`Tooltip`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct TooltipPlugin;

impl Plugin for TooltipPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(despawn_popup_with_owner)
            .add_observer(track_touch)
            .add_systems(
                Update,
                (
                    track_active_tooltips,
                    update_tooltips,
                    place_popups,
                    restyle_popups.run_if(resource_changed::<Theme>),
                )
                    .chain(),
            );
    }
}

/// A short text shown in a popup below the node after the pointer has rested on it for
/// [`TOOLTIP_DELAY_SECS`]. A finger that is held on the node for as long, without moving, shows
/// it too (see below).
///
/// The popup is a [`Float`] child of the node, placed in the viewport below the node and kept
/// inside the window, and goes above it when it would run off the bottom. It hides when the
/// pointer leaves the node or presses it (and does not come back until the pointer has left
/// and returned), and it never takes the pointer. An
/// [`IconButton`](crate::icon_button::IconButton) has one with its name, and so has the button
/// of an underlined [`Toolbar`](crate::toolbar::Toolbar), whose labels are hidden.
///
/// A finger held on the node (or a descendant) for [`TOOLTIP_DELAY_SECS`] without moving past a
/// few pixels shows the popup above the node when there is room (the finger covers what is
/// below), else below it. The popup goes when the finger lifts or the touch is cancelled. A long
/// press that showed one does not press the widget: its pressed look ends when the popup shows
/// and the lift activates nothing (no [`Activate`](bevy::ui_widgets::Activate), pick or value
/// change). A finger that moves before the delay, or a short tap, shows none. Only the first
/// finger counts.
///
/// A `Tooltip` with empty text shows no popup (an empty bordered box would say nothing); a popup
/// that is showing goes when the text is changed to empty.
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
    /// The finger that is down on the node, if any.
    touch: Option<TouchPress>,
}

/// A finger that went down on a [`Tooltip`] node and has not moved past the drag threshold.
#[derive(Clone, Copy, Debug, PartialEq)]
struct TouchPress {
    /// The finger; the first one wins, others are ignored.
    pointer: PointerId,
    /// The entity the finger went down on: the node or one of its descendants.
    target: Entity,
    /// Seconds the finger has been down without moving.
    elapsed: f32,
}

/// Marks a [`Tooltip`] node that `update_tooltips` must visit: the pointer is on it, or it has
/// state to wind down. Idle tooltips have none, so a quiet frame visits nothing.
#[derive(Component, Clone, Copy, Debug)]
struct TooltipActive;

/// The popup of a [`Tooltip`] node that is showing.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub struct TooltipPopup {
    /// The node the popup belongs to.
    owner: Entity,
}

impl TooltipPopup {
    /// The node the popup belongs to.
    #[must_use]
    pub const fn owner(&self) -> Entity {
        self.owner
    }
}

/// The text of a popup.
#[derive(Component, Clone, Copy, Debug)]
struct TooltipText;

/// Marks the popup of a long press: it is placed above the node when there is room.
#[derive(Component, Clone, Copy, Debug)]
struct TouchPopup;

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

/// Marks the tooltip nodes the pointer has come onto, when `Hovered` or the `Tooltip` changes.
fn track_active_tooltips(
    changed: Query<
        (Entity, &Hovered),
        (
            With<Tooltip>,
            Without<TooltipActive>,
            Or<(Changed<Hovered>, Changed<Tooltip>)>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, hovered) in &changed {
        if hovered.get() {
            commands.entity(entity).insert(TooltipActive);
        }
    }
}

/// Follows a finger on a [`Tooltip`] node: a press starts counting, a move past the drag
/// threshold before the popup shows, a lift, a cancel or the finger leaving ends it.
fn track_touch(
    t: On<MouseEvent>,
    parents: Query<&ChildOf>,
    tooltips: Query<(), With<Tooltip>>,
    mut clocks: ParamSet<(
        Query<&mut TooltipClock>,
        Query<(&mut TooltipClock, Has<TooltipRef>), With<TooltipActive>>,
    )>,
    mut commands: Commands,
) {
    let event = t.event();
    if !event.touch {
        return;
    }
    if matches!(event.kind, MouseKind::Pressed) {
        let node = std::iter::successors(Some(event.entity), |e| {
            parents.get(*e).ok().map(ChildOf::parent)
        })
        .find(|e| tooltips.contains(*e));
        if let Some(node) = node
            && let Ok(mut clock) = clocks.p0().get_mut(node)
            && clock.touch.is_none()
        {
            clock.touch = Some(TouchPress {
                pointer: event.pointer,
                target: event.entity,
                elapsed: 0.0,
            });
            commands.entity(node).insert(TooltipActive);
        }
        return;
    }
    for (mut clock, shown) in &mut clocks.p1() {
        let Some(touch) = clock.touch else {
            continue;
        };
        if touch.pointer != event.pointer {
            continue;
        }
        let ends = match event.kind {
            MouseKind::Released | MouseKind::Cancel => true,
            MouseKind::Out => event.entity == touch.target,
            MouseKind::Drag => !shown && event.distance.length() > DRAG_THRESHOLD,
            _ => false,
        };
        if ends {
            clock.touch = None;
        }
    }
}

fn update_tooltips(
    time: Res<Time>,
    mut nodes: Query<
        (
            Entity,
            &Tooltip,
            &Hovered,
            Has<Pressed>,
            &mut TooltipClock,
            Option<&TooltipRef>,
        ),
        With<TooltipActive>,
    >,
    mut popup_texts: Query<&mut Text, With<TooltipText>>,
    popups: Query<&Children, With<TooltipPopup>>,
    theme: Res<Theme>,
    hover_lock: Option<Res<HoverLock>>,
    parents: Query<&ChildOf>,
    mut commands: Commands,
) {
    let locked = hover_lock.is_some_and(|lock| lock.0);
    for (entity, tooltip, hovered, pressed, mut clock, popup) in &mut nodes {
        if !hovered.get() {
            if clock.elapsed != 0.0 || clock.suppressed {
                clock.elapsed = 0.0;
                clock.suppressed = false;
            }
        } else if locked {
            // A scrub drag holds the pointer: no tooltip shows or starts counting.
            clock.elapsed = 0.0;
        } else if pressed {
            clock.suppressed = true;
            clock.elapsed = 0.0;
        } else if !clock.suppressed {
            clock.elapsed += time.delta_secs();
        }
        if let Some(touch) = clock.touch.as_mut() {
            // A scrub drag holds the pointer: no touch tooltip shows or counts either.
            touch.elapsed = if locked {
                0.0
            } else {
                touch.elapsed + time.delta_secs()
            };
        }
        let touched = clock
            .touch
            .is_some_and(|touch| touch.elapsed >= TOOLTIP_DELAY_SECS);
        let wanted = !locked
            && !tooltip.text.is_empty()
            && (touched
                || (hovered.get() && !clock.suppressed && clock.elapsed >= TOOLTIP_DELAY_SECS));
        match (wanted, popup) {
            (true, None) => {
                let popup = spawn_popup(&mut commands, &theme, entity, &tooltip.text);
                commands.entity(entity).insert(TooltipRef(popup));
                if let Some(touch) = clock.touch.filter(|_| touched) {
                    commands.entity(popup).insert(TouchPopup);
                    end_press(&mut commands, &parents, touch, entity);
                }
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
        if !hovered.get() && clock.touch.is_none() && !wanted && popup.is_none() {
            // Nothing left to wind down: stop visiting until the pointer returns.
            commands.entity(entity).remove::<TooltipActive>();
        }
    }
}

/// Ends the press a long press began, from the node it went down on up to the tooltip node: the
/// buttons among them lose `Pressed`, so the lift fires no `Activate` (Bevy's button needs it),
/// and a `Cancel` restores the resting look as a cancelled touch does (hover is false for it).
fn end_press(commands: &mut Commands, parents: &Query<&ChildOf>, touch: TouchPress, node: Entity) {
    let mut entity = touch.target;
    loop {
        commands.entity(entity).try_remove::<Pressed>();
        commands.trigger(MouseEvent {
            entity,
            kind: MouseKind::Cancel,
            distance: Vec2::ZERO,
            button: None,
            // Not marked as a finger's, so `track_touch` does not take it for the finger
            // going away: the popup stays until the finger lifts.
            touch: false,
            pointer: PointerId::Mouse,
        });
        if entity == node {
            break;
        }
        let Ok(parent) = parents.get(entity) else {
            break;
        };
        entity = parent.parent();
    }
}

fn spawn_popup(commands: &mut Commands, theme: &Theme, owner: Entity, text: &str) -> Entity {
    let popup = commands
        .spawn((
            Name::new("Tooltip popup"),
            TooltipPopup { owner },
            Float,
            popup_node(theme),
            BackgroundColor(theme.colors().card_surface),
            BorderColor::all(theme.colors().border),
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
        TextColor(theme.colors().text_primary),
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
        .try_remove::<(TooltipRef, TooltipActive)>()
        .try_insert(TooltipClock::default());
}

/// Puts each popup centred under its node, inside the window, or above the node when it would
/// run off the bottom, and shows it. Needs the popup's laid-out size, so it shows one frame
/// after it is spawned.
fn place_popups(
    viewport: UiViewport,
    mut popups: Query<(
        &TooltipPopup,
        &ComputedNode,
        &mut Node,
        &mut Visibility,
        Has<TouchPopup>,
    )>,
    owners: Query<(&ComputedNode, &UiGlobalTransform), Without<TooltipPopup>>,
) {
    for (popup, popup_computed, mut node, mut visibility, touch) in &mut popups {
        let Ok((owner_computed, owner_transform)) = owners.get(popup.owner()) else {
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
        let above = center.y - owner_size.y / 2.0 - GAP - popup_size.y;
        let target = viewport.size_of(popup.owner());
        if let Some(target) = target {
            left = left.min(target.x - MARGIN - popup_size.x).max(MARGIN);
        }
        let fits_below = target.is_none_or(|t| below + popup_size.y + MARGIN <= t.y);
        // A finger covers what is below the node, so a touch popup goes above it when there is
        // room; any popup goes above when it would run off the bottom, but never above the
        // window's margin, whatever room there is.
        let top = if touch && above >= MARGIN {
            above
        } else if fits_below {
            below
        } else {
            above.max(MARGIN)
        };
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
        background.0 = theme.colors().card_surface;
        *border = BorderColor::all(theme.colors().border);
        for child in children {
            if let Ok((mut font, mut color)) = texts.get_mut(*child) {
                font.set_if_neq(text_font(&theme));
                color.set_if_neq(TextColor(theme.colors().text_primary));
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::disallowed_methods,
    reason = "tests may abort loudly"
)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<Theme>()
            .add_observer(despawn_popup_with_owner)
            .add_observer(track_touch)
            .add_systems(Update, (track_active_tooltips, update_tooltips).chain());
        app.finish();
        app
    }

    fn tick(app: &mut App) {
        app.world_mut()
            .resource_mut::<Time>()
            .advance_by(Duration::from_millis(200));
        app.world_mut().run_schedule(Update);
    }

    fn active(app: &mut App) -> usize {
        let world = app.world_mut();
        world
            .query_filtered::<(), With<TooltipActive>>()
            .iter(world)
            .count()
    }

    #[test]
    fn only_a_hovered_or_showing_tooltip_is_visited() {
        let mut app = app();
        let nodes: Vec<Entity> = (0..10)
            .map(|_| app.world_mut().spawn(Tooltip::new("Tip")).id())
            .collect();
        for _ in 0..3 {
            tick(&mut app);
        }
        assert_eq!(active(&mut app), 0, "idle tooltips are visited");

        let hovered = *nodes.first().unwrap();
        app.world_mut().entity_mut(hovered).insert(Hovered(true));
        tick(&mut app);
        assert_eq!(active(&mut app), 1, "only the hovered tooltip is marked");
        for _ in 0..4 {
            tick(&mut app);
        }
        assert!(app.world().get::<TooltipRef>(hovered).is_some());

        app.world_mut().entity_mut(hovered).insert(Hovered(false));
        tick(&mut app);
        tick(&mut app);
        assert!(app.world().get::<TooltipRef>(hovered).is_none());
        assert_eq!(active(&mut app), 0, "the marker outlives the pointer");
    }

    /// A tooltip nobody is on is never visited. Its clock is given a value that `update_tooltips`
    /// would reset (an unhovered node's clock is always the default), so a visit shows as a
    /// changed value and as a write.
    #[test]
    fn an_unmarked_tooltip_is_not_visited_in_a_quiet_frame() {
        let mut app = app();
        let stale = TooltipClock {
            elapsed: 0.3,
            suppressed: true,
            touch: None,
        };
        let node = app.world_mut().spawn((Tooltip::new("Tip"), stale)).id();
        tick(&mut app);
        let written = |app: &App| {
            app.world()
                .entity(node)
                .get_ref::<TooltipClock>()
                .unwrap()
                .last_changed()
        };
        let before = written(&app);
        for _ in 0..3 {
            tick(&mut app);
        }
        assert_eq!(
            app.world().get::<TooltipClock>(node),
            Some(&stale),
            "an idle tooltip was visited"
        );
        assert_eq!(written(&app), before, "an idle tooltip's clock was written");
    }

    fn touch_event(app: &mut App, entity: Entity, kind: MouseKind, distance: Vec2) {
        app.world_mut().trigger(MouseEvent {
            entity,
            kind,
            distance,
            button: Some(PointerButton::Primary),
            touch: true,
            pointer: PointerId::Touch(0),
        });
        app.world_mut().flush();
    }

    #[test]
    fn a_touch_is_visited_only_while_it_is_down_or_its_popup_shows() {
        let mut app = app();
        let nodes: Vec<Entity> = (0..5)
            .map(|_| app.world_mut().spawn(Tooltip::new("Tip")).id())
            .collect();
        let node = *nodes.first().unwrap();
        tick(&mut app);
        assert_eq!(active(&mut app), 0);

        touch_event(&mut app, node, MouseKind::Pressed, Vec2::ZERO);
        assert_eq!(active(&mut app), 1, "only the touched tooltip is marked");
        tick(&mut app);
        assert!(app.world().get::<TooltipRef>(node).is_none());
        for _ in 0..2 {
            tick(&mut app);
        }
        assert!(
            app.world().get::<TooltipRef>(node).is_some(),
            "no long press popup"
        );

        touch_event(&mut app, node, MouseKind::Released, Vec2::ZERO);
        tick(&mut app);
        tick(&mut app);
        assert!(app.world().get::<TooltipRef>(node).is_none());
        assert_eq!(active(&mut app), 0, "the marker outlives the touch");
    }

    #[test]
    fn a_short_tap_or_a_moving_touch_shows_no_popup() {
        let mut app = app();
        let node = app.world_mut().spawn(Tooltip::new("Tip")).id();
        touch_event(&mut app, node, MouseKind::Pressed, Vec2::ZERO);
        tick(&mut app);
        touch_event(&mut app, node, MouseKind::Released, Vec2::ZERO);
        for _ in 0..4 {
            tick(&mut app);
        }
        assert!(app.world().get::<TooltipRef>(node).is_none(), "a short tap");
        assert_eq!(active(&mut app), 0);

        touch_event(&mut app, node, MouseKind::Pressed, Vec2::ZERO);
        tick(&mut app);
        touch_event(&mut app, node, MouseKind::Drag, Vec2::new(2.0, 0.0));
        touch_event(&mut app, node, MouseKind::Drag, Vec2::new(4.0, 0.0));
        for _ in 0..4 {
            tick(&mut app);
        }
        assert!(
            app.world().get::<TooltipRef>(node).is_none(),
            "a moving finger"
        );
        assert_eq!(active(&mut app), 0);
    }

    #[test]
    fn only_the_first_finger_counts_and_a_cancel_ends_it() {
        let mut app = app();
        let node = app.world_mut().spawn(Tooltip::new("Tip")).id();
        touch_event(&mut app, node, MouseKind::Pressed, Vec2::ZERO);
        app.world_mut().trigger(MouseEvent {
            entity: node,
            kind: MouseKind::Released,
            distance: Vec2::ZERO,
            button: Some(PointerButton::Primary),
            touch: true,
            pointer: PointerId::Touch(1),
        });
        for _ in 0..4 {
            tick(&mut app);
        }
        assert!(app.world().get::<TooltipRef>(node).is_some());
        touch_event(&mut app, node, MouseKind::Cancel, Vec2::ZERO);
        tick(&mut app);
        tick(&mut app);
        assert!(app.world().get::<TooltipRef>(node).is_none());
        assert_eq!(active(&mut app), 0);
    }
}
