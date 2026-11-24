//! `ScrollArea`: scrollable contents with vertical and/or horizontal scroll bars.
//!
//! The scroll bars are `bevy_ui_widgets` [`Scrollbar`]s: the widget sizes and places
//! each thumb, scrolls the contents when the thumb is dragged and pages them on a click
//! on the track. This module keeps the layout, the look, the wheel and scrolling a
//! focused row into view.

use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::template::TemplateContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::{HoverMap, Hovered};
use bevy::picking::pointer::{PointerButton, PointerId};
use bevy::prelude::*;
use bevy::scene::EmptySceneEffect;
use bevy::ui::{FixedNode, Pressed, UiSystems};
use bevy::ui_widgets::{
    ControlOrientation, ScrollIntoView, Scrollbar, ScrollbarDragState, ScrollbarThumb,
};
use itertools::Either;

use crate::focus::FOCUS_RING_REACH;
use crate::numeric_stepper::NumericStepper;
use crate::slider::Slider;
use crate::theme::{Theme, ThemeColors};
use crate::{DRAG_THRESHOLD, MouseEvent, MouseKind};

/// Registers the systems and observers of [`ScrollArea`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct ScrollAreaPlugin;

impl Plugin for ScrollAreaPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TouchPans>()
            .add_observer(scroll_into_view)
            .add_observer(pan_with_finger)
            .add_systems(Update, update_scroll_position)
            .add_systems(Update, step_flings.run_if(any_with_component::<Fling>))
            .add_systems(
                Update,
                end_caught_presses.run_if(|pans: Res<TouchPans>| !pans.caught.is_empty()),
            )
            .add_systems(
                PostUpdate,
                (
                    refresh_changed_scroll_bars.before(UiSystems::Layout),
                    show_scroll_bars,
                    restyle_thumbs_on_drag_change,
                    place_popup_scroll_bars.before(UiSystems::Layout),
                ),
            )
            .add_systems(
                Update,
                (
                    (|| None::<Entity>)
                        .pipe(theme_popup_scroll_bars)
                        .run_if(resource_changed::<Theme>),
                    theme_new_popup_scroll_bars,
                ),
            )
            .add_systems(
                Update,
                (|| None::<Entity>)
                    .pipe(theme_scroll_area)
                    .run_if(resource_changed::<Theme>),
            )
            .add_systems(
                Update,
                (|| None::<Entity>)
                    .pipe(clear_of_corners)
                    .run_if(resource_changed::<Theme>),
            )
            .register_type::<ClearOfCorners>()
            .register_type::<ScrollBar>()
            .register_type::<ScrollAreaRefs>();
    }
}

/// A node with scroll bars. Build one with [`ScrollArea::bundle`], or with [`ScrollArea::scene`]
/// in a `bsn!` scene.
///
/// To assistive technology it is a scroll view ([`Role::ScrollView`]); its scroll bars and
/// their thumbs are not in the accessibility tree.
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[require(
    Node::default(),
    Name::new("ScrollArea"),
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::ScrollView)),
)]
#[component(on_insert=Self::on_insert)]
pub struct ScrollArea;

impl ScrollArea {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let mut entity = world.entity_mut(context.entity);
        let Some(mut node) = entity.get_mut::<Node>() else {
            return;
        };

        node.display = Display::Grid;
        node.grid_template_columns =
            vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)];
        node.grid_template_rows = vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)];
    }
}

/// Opt-in that keeps a [`ScrollArea`] at the bottom of a rounded [`Card`](crate::card::Card)
/// above the card's corners.
///
/// The area gets a bottom margin of the card's corner radius (`Theme::panel_radius`).
///
/// An area clips to a rectangle (Bevy 0.20 has no rounded clip), so contents scrolled half out
/// of view, or with square corners of their own, would otherwise stick out of the card's rounded
/// bottom corners. It is opt-in because an area cannot tell whether it touches the card's
/// corners (other nodes may sit below it, or the card may be square), and it keeps
/// the margin in step with the theme. Insert it with the area, or later. A [`Panel`](crate::panel::Panel)
/// does the same for its own body, and only while it has no footer. Removing it sets the bottom margin back to zero.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
#[component(on_add=Self::on_add, on_remove=Self::on_remove)]
pub struct ClearOfCorners;

impl ClearOfCorners {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(clear_of_corners, Some(context.entity));
    }

    /// Takes the margin back to [`Val::ZERO`] when the opt-in goes.
    fn on_remove(mut world: DeferredWorld, context: HookContext) {
        if let Some(mut node) = world.get_mut::<Node>(context.entity) {
            node.margin.bottom = Val::ZERO;
        }
    }
}

fn clear_of_corners(
    area: In<Option<Entity>>,
    theme: Res<Theme>,
    mut areas: Query<&mut Node, With<ClearOfCorners>>,
) {
    let iter = if let Some(area) = area.0 {
        Either::Left(areas.get_mut(area).into_iter())
    } else {
        Either::Right(areas.iter_mut())
    };
    for mut node in iter {
        if node.margin.bottom != theme.panel_radius {
            node.margin.bottom = theme.panel_radius;
        }
    }
}

/// Marks the node holding the scrolled contents of a [`ScrollArea`].
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ScrollContents;

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct ScrollAreaRefs {
    v_scrollbar: Option<Entity>,
    h_scrollbar: Option<Entity>,
}

impl ScrollArea {
    /// A scroll area whose contents are spawned by `contents_spawner`.
    ///
    /// `v_scroll` and `h_scroll` enable scrolling (and a scroll bar) on that axis.
    pub fn bundle<F>(contents_spawner: F, v_scroll: bool, h_scroll: bool) -> impl Bundle
    where
        F: FnOnce(&mut ChildSpawner) + Send + Sync + 'static,
    {
        let overflow = Overflow {
            x: if h_scroll {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            },
            y: if v_scroll {
                OverflowAxis::Scroll
            } else {
                OverflowAxis::Hidden
            },
        };
        (
            Self,
            Children::spawn(SpawnWith(move |s: &mut ChildSpawner| {
                let contents_ref = s
                    .spawn((
                        ScrollContents,
                        Children::spawn(SpawnWith(contents_spawner)),
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            overflow,
                            // Place the contents explicitly: auto-placement puts a
                            // bar in the first free cell, which is not its own when
                            // the area lacks the other bar.
                            grid_row: GridPlacement::start(1),
                            grid_column: GridPlacement::start(1),
                            // Keep children at their natural size: a child stretched to
                            // this node's height reported no overflow when it is a grid
                            // (taffy 0.10, Bevy 0.19), which left nothing to scroll; still
                            // needed on taffy 0.14: `Stretch` leaves the maximum scroll 2 px short (#265).
                            align_items: AlignItems::FlexStart,
                            // The area clips at its padding box, so the focus ring of
                            // a child at the edge (2 px wide, 1 px offset) needs room
                            // inside it (#293).
                            padding: UiRect::all(Val::Px(FOCUS_RING_REACH)),
                            ..default()
                        },
                    ))
                    .id();

                let mut spawn_bar = |orientation| {
                    s.spawn((ScrollBar, Scrollbar::new(contents_ref, orientation, 0.0)))
                        .id()
                };
                let refs = ScrollAreaRefs {
                    v_scrollbar: v_scroll.then(|| spawn_bar(ControlOrientation::Vertical)),
                    h_scrollbar: h_scroll.then(|| spawn_bar(ControlOrientation::Horizontal)),
                };
                s.world_mut().entity_mut(contents_ref).insert(refs);
                s.world_mut()
                    .commands()
                    .run_system_cached_with(theme_scroll_area, Some(contents_ref));
            })),
        )
    }

    /// A scroll area for a `bsn!` scene, whose contents the scene list `contents()` makes.
    ///
    /// Use it as an entry of a scene (`@scroll_area(..)` of a function returning it, or by
    /// composing it into another scene). `v_scroll` and `h_scroll` enable scrolling (and a
    /// scroll bar) on that axis. It builds the same area as [`ScrollArea::bundle`], which stays
    /// for code that spawns through `Commands`.
    ///
    /// The factory runs for every build of the template, so each area gets fresh contents of
    /// its own, also when a resolved scene is instantiated again. Clone what the contents
    /// capture (a handle, say) inside it:
    ///
    /// ```ignore
    /// ScrollArea::scene(move || bsn_list! { @squares(icon.clone()) }, true, false)
    /// ```
    pub fn scene<L: SceneList>(
        contents: impl Fn() -> L + Send + Sync + 'static,
        v_scroll: bool,
        h_scroll: bool,
    ) -> impl Scene {
        // A scene's template must be `Clone`, a scene list is not: share the factory instead.
        let contents = Arc::new(contents);
        bsn! {
            template(move |context| {
                let contents = Arc::clone(&contents);
                Ok(insert_after_scene(context, Self::bundle(
                    move |parent: &mut ChildSpawner| {
                        let target = parent.target_entity();
                        parent
                            .world_mut()
                            .entity_mut(target)
                            .queue_spawn_related_scenes::<Children>(contents());
                    },
                    v_scroll,
                    h_scroll,
                )))
            })
        }
    }
}

/// Inserts `bundle` on the entity of a scene's template once the scene has been applied, so
/// that the scene's own `Node` does not overwrite what the area's hooks set and the area is
/// not taken for a root before it has a parent.
fn insert_after_scene(context: &mut TemplateContext, bundle: impl Bundle) -> EmptySceneEffect {
    let entity = context.entity.id();
    context.entity.world_scope(|world| {
        world.commands().entity(entity).insert(bundle);
    });
    EmptySceneEffect
}

/// Marks a scroll bar of a [`ScrollArea`]: a node with exactly one thumb, placed in the grid
/// cell of its axis and themed.
///
/// The axis is not stored here: it is the `orientation` of the `bevy_ui_widgets`
/// [`Scrollbar`] on the same entity, which also names the contents it scrolls.
/// [`ScrollArea::bundle`] spawns both. The marker is not a scrolling widget on its own, a bar
/// without a [`Scrollbar`] does nothing. Inserting the marker again reuses the existing thumb
/// and places the bar again; changing the `Scrollbar`'s orientation on a live bar moves the bar
/// to the other cell and swaps its thickness on the next frame.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert = Self::on_insert)]
#[require(Node::default(), Visibility::Inherited, Name::new("ScrollBar"))]
pub struct ScrollBar;

impl ScrollBar {
    /// Places the bar by its [`Scrollbar`] and queues the setup of its thumb as one idempotent
    /// command: two inserts before the commands are applied still leave one thumb.
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let bar = context.entity;
        let orientation = world
            .get::<Scrollbar>(bar)
            .map(|scrollbar| scrollbar.orientation);
        if let (Some(orientation), Some(mut node)) = (orientation, world.get_mut::<Node>(bar)) {
            place(&mut node, orientation);
        }
        world.commands().queue(move |world: &mut World| {
            let has_thumb = world.get::<Children>(bar).is_some_and(|children| {
                children
                    .iter()
                    .any(|child| world.get::<ScrollThumb>(child).is_some())
            });
            if !has_thumb && world.get_entity(bar).is_ok() {
                world
                    .spawn((ScrollThumb, ChildOf(bar)))
                    .observe(handle_hover);
            }
        });
    }
}

/// Puts a bar in the grid cell of its axis: the vertical bar right of the contents, the
/// horizontal bar below them, whichever bars the area has.
fn place(node: &mut Node, orientation: ControlOrientation) {
    let (row, column) = match orientation {
        ControlOrientation::Horizontal => (2, 1),
        ControlOrientation::Vertical => (1, 2),
    };
    node.grid_row = GridPlacement::start(row);
    node.grid_column = GridPlacement::start(column);
}

/// Sets the thickness of a bar across its axis, and lets the other dimension size itself.
const fn size_bar(node: &mut Node, orientation: ControlOrientation, thickness: Val) {
    match orientation {
        ControlOrientation::Horizontal => {
            node.height = thickness;
            node.width = Val::Auto;
        }
        ControlOrientation::Vertical => {
            node.width = thickness;
            node.height = Val::Auto;
        }
    }
}

/// Places and sizes a bar again when its [`Scrollbar`]'s orientation was changed.
fn refresh_changed_scroll_bars(
    theme: Res<Theme>,
    mut bars: Query<(&Scrollbar, &mut Node), (With<ScrollBar>, Changed<Scrollbar>)>,
) {
    for (scrollbar, mut node) in &mut bars {
        let mut placed = node.clone();
        place(&mut placed, scrollbar.orientation);
        size_bar(&mut placed, scrollbar.orientation, theme.scroll_bar_width);
        node.set_if_neq(placed);
    }
}

/// The thumb of a [`ScrollBar`]: a `bevy_ui_widgets` [`ScrollbarThumb`], which has no
/// `Node` because the widget sizes and places it along its track.
#[derive(Component, Clone, Copy, Debug)]
#[require(ScrollbarThumb, Hovered, Pickable::default(), Name::new("ScrollThumb"))]
struct ScrollThumb;

/// The thumb's background: the one function of its state that the pointer handler, the drag
/// restyle and `theme_scroll_area` all call, so a theme change under the pointer keeps the colour.
const fn thumb_background(colors: &ThemeColors, hovered: bool, pressed: bool) -> Color {
    if pressed {
        colors.selected_hovered_item
    } else if hovered {
        colors.hovered_item
    } else {
        colors.scroll_thumb_background
    }
}

fn handle_hover(
    trigger: On<MouseEvent>,
    mut query: Query<(&mut BackgroundColor, Option<&ScrollbarDragState>)>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    // A click or a drag says nothing new: the events around them set the state.
    let kind = trigger.event().kind;
    if matches!(kind, MouseKind::Click(_) | MouseKind::Drag) {
        return;
    }
    let thumb = trigger.event().event_target();
    let Ok((mut background, drag)) = query.get_mut(thumb) else {
        return;
    };
    // The pointer is on the thumb for every event but `Out`, a release included.
    let hovered = trigger.event().hovered();
    let dragging = drag.is_some_and(|drag| drag.dragging);
    let pressed = trigger.event().pressed() || (dragging && hovered);
    if pressed {
        commands.entity(thumb).insert(Pressed);
    } else {
        commands.entity(thumb).remove::<Pressed>();
    }
    *background = BackgroundColor(thumb_background(theme.colors(), hovered, pressed));
}

/// Restyles a thumb whose drag started or ended: a drag that ends with the pointer away from the
/// thumb sends it no release, so the pointer events alone would leave the pressed colour.
fn restyle_thumbs_on_drag_change(
    mut thumbs: Query<
        (
            &ScrollbarDragState,
            &Hovered,
            &mut BackgroundColor,
            Has<Pressed>,
        ),
        (With<ScrollThumb>, Changed<ScrollbarDragState>),
    >,
    theme: Res<Theme>,
) {
    for (drag, hovered, mut background, pressed) in &mut thumbs {
        let color = thumb_background(theme.colors(), hovered.get(), pressed || drag.dragging);
        background.set_if_neq(BackgroundColor(color));
    }
}

/// Scrolls the [`ScrollContents`] around an entity just far enough to show it and its focus ring.
///
/// Answers [`ScrollIntoView`], which a `ListBox` triggers when the arrow keys
/// move its active descendant. An axis is moved only when the contents scroll
/// along it, and the [`ScrollPosition`] stays within `0..=max` as in
/// `update_scroll_position`. An entity outside any scroll area is left alone. One in a
/// [`FixedNode`] (a popup) scrolls that popup when it scrolls (a `Combo` list capped to the
/// window height) and otherwise nothing, never the area around it: the popup is not laid
/// out in the area's contents, so scrolling them would not show it.
fn scroll_into_view(
    mut event: On<ScrollIntoView>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    parents: Query<&ChildOf>,
    fixed_nodes: Query<(), With<FixedNode>>,
    mut contents: Query<(&mut ScrollPosition, &Node), Or<(With<ScrollContents>, With<FixedNode>)>>,
) {
    let target = event.entity;
    let Ok((target_computed, target_transform)) = nodes.get(target) else {
        return;
    };
    let Some(area) = parents
        .iter_ancestors(target)
        .find(|ancestor| contents.contains(*ancestor))
    else {
        return;
    };
    let Ok((area_computed, area_transform)) = nodes.get(area) else {
        return;
    };
    let Ok((mut scroll_position, node)) = contents.get_mut(area) else {
        return;
    };
    // A popup that does not scroll is not laid out in the contents of the area around it, so
    // scrolling those would not show the target.
    if fixed_nodes.contains(area)
        && node.overflow.x != OverflowAxis::Scroll
        && node.overflow.y != OverflowAxis::Scroll
    {
        return;
    }
    event.propagate(false);

    let target_scale = target_computed.inverse_scale_factor();
    let area_scale = area_computed.inverse_scale_factor();
    let target_size = target_computed.size() * target_scale;
    let area_size = area_computed.size() * area_scale;
    // Top left corners in logical pixels, as the layout last placed them (the
    // target's already shifted by the scroll position it had then).
    let target_top_left = target_transform.translation * target_scale - target_size * 0.5;
    let area_top_left = area_transform.translation * area_scale - area_size * 0.5;
    // The target's focus ring is shown with it, so its reach counts as part of the target.
    let ring = Vec2::splat(FOCUS_RING_REACH);
    let local_top_left = target_top_left - area_top_left + scroll_position.0 - ring;
    let local_bottom_right = local_top_left + target_size + ring * 2.0;
    let max = ((area_computed.content_size() - area_computed.size()) * area_scale).max(Vec2::ZERO);

    if node.overflow.x == OverflowAxis::Scroll {
        let view = scroll_position.x;
        if local_top_left.x < view {
            scroll_position.x = local_top_left.x.clamp(0.0, max.x);
        } else if local_bottom_right.x > view + area_size.x {
            scroll_position.x = (local_bottom_right.x - area_size.x).clamp(0.0, max.x);
        }
    }
    if node.overflow.y == OverflowAxis::Scroll {
        let view = scroll_position.y;
        if local_top_left.y < view {
            scroll_position.y = local_top_left.y.clamp(0.0, max.y);
        } else if local_bottom_right.y > view + area_size.y {
            scroll_position.y = (local_bottom_right.y - area_size.y).clamp(0.0, max.y);
        }
    }
}

/// Scrolls the nodes under the pointer by the mouse wheel.
///
/// An axis is moved only when the node's [`Node::overflow`] scrolls along it,
/// and the [`ScrollPosition`] stays within `0..=max` on that axis, where `max`
/// is how far the contents overflow the node (0 when they fit). Both are in
/// logical pixels. It moves any [`ScrollPosition`] under the pointer, not only a
/// [`ScrollContents`], so a node of an app's own with one scrolls too.
///
/// With Ctrl held the wheel's horizontal and vertical deltas are swapped first.
/// Then a node that scrolls horizontally but not vertically (`overflow.y` is not
/// `Scroll`) takes a wheel event with no horizontal delta (`dx == 0`) as a
/// horizontal scroll by its vertical delta, since a mouse wheel only turns
/// vertically. An event with a horizontal delta scrolls by that delta alone.
///
/// The nearest scrollable ancestor of each hovered entity receives the wheel. Nested
/// areas scroll only the innermost area, even when it is at its limit. Fixed nodes
/// stop the ancestor search so a floating popup does not scroll its owner.
///
/// Registered once, by [`ScrollAreaPlugin`].
pub(crate) fn update_scroll_position(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<(&mut ScrollPosition, &ComputedNode, &Node)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    hierarchy: Query<(Option<&ChildOf>, Has<FixedNode>)>,
    flinging: Query<(), With<Fling>>,
    mut commands: Commands,
    mut scratch: Local<WheelScratch>,
) {
    if mouse_wheel_events.is_empty() {
        return;
    }
    // The hover map and the hierarchy do not change while this system runs, so the
    // scroll targets are the same for every wheel message of the frame: find them once.
    let WheelScratch {
        targets,
        outer_targets,
        resolved,
    } = &mut *scratch;
    resolved.clear();
    for pointer_map in hover_map.values() {
        targets.clear();
        for entity in pointer_map.keys() {
            let mut current = Some(*entity);
            while let Some(entity) = current {
                if let Ok((_, _, node)) = scrolled_node_query.get(entity)
                    && (node.overflow.x == OverflowAxis::Scroll
                        || node.overflow.y == OverflowAxis::Scroll)
                {
                    targets.insert(entity);
                    break;
                }
                let Ok((parent, fixed)) = hierarchy.get(entity) else {
                    break;
                };
                if fixed {
                    break;
                }
                current = parent.map(ChildOf::parent);
            }
        }
        // A nested scroll area gets the wheel alone, even if picking also reports
        // its outer area. Do not pass the wheel on when the inner area is at its limit.
        outer_targets.clear();
        for &target in targets.iter() {
            let mut current = hierarchy.get(target).ok().and_then(|(parent, fixed)| {
                if fixed {
                    None
                } else {
                    parent.map(ChildOf::parent)
                }
            });
            while let Some(entity) = current {
                if targets.contains(&entity) {
                    outer_targets.insert(entity);
                }
                let Ok((parent, fixed)) = hierarchy.get(entity) else {
                    break;
                };
                if fixed {
                    break;
                }
                current = parent.map(ChildOf::parent);
            }
        }
        // One entry per pointer, so an area two pointers hover scrolls once for each.
        resolved.extend(targets.difference(outer_targets).copied());
    }

    let ctrl = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (mouse_wheel_event.x * 21., mouse_wheel_event.y * 21.),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if ctrl {
            std::mem::swap(&mut dx, &mut dy);
        }

        // Each message is applied in turn (and clamped in turn), as before.
        for entity in resolved.iter() {
            let Ok((mut scroll_position, computed, node)) = scrolled_node_query.get_mut(*entity)
            else {
                continue;
            };
            // A mouse wheel only turns vertically, so an area that scrolls sideways
            // alone takes the vertical turn for a horizontal one.
            let dx = if dx == 0.0 && node.overflow.y != OverflowAxis::Scroll {
                dy
            } else {
                dx
            };
            // A turn of the wheel stops a fling, even one the area cannot move any further.
            if flinging.contains(*entity) {
                commands.entity(*entity).try_remove::<Fling>();
            }
            scroll_by(&mut scroll_position, computed, node, dx, dy);
        }
    }
}

/// Scrolls `scroll_position` by `(-dx, -dy)` logical pixels along the axes `node` scrolls,
/// keeping each in `[0, max]`, `max` being how far the contents overflow.
///
/// Writes only a position that moves: a mutable dereference marks the `ScrollPosition`
/// changed, which wakes its observers (a combo closing on an ancestor's scroll) although
/// nothing scrolled. Shared by the wheel and a finger's pan.
fn scroll_by(
    scroll_position: &mut Mut<ScrollPosition>,
    computed: &ComputedNode,
    node: &Node,
    dx: f32,
    dy: f32,
) {
    let overflow = max_scroll(computed);
    let new_x = (node.overflow.x == OverflowAxis::Scroll)
        .then(|| (scroll_position.x - dx).clamp(0.0, overflow.x));
    let new_y = (node.overflow.y == OverflowAxis::Scroll)
        .then(|| (scroll_position.y - dy).clamp(0.0, overflow.y));
    if new_x.is_some_and(|x| x.to_bits() != scroll_position.x.to_bits())
        || new_y.is_some_and(|y| y.to_bits() != scroll_position.y.to_bits())
    {
        if let Some(x) = new_x {
            scroll_position.x = x;
        }
        if let Some(y) = new_y {
            scroll_position.y = y;
        }
    }
}

/// How far the contents of `computed` overflow it, in logical pixels: the largest
/// [`ScrollPosition`] on each axis (0 when they fit).
fn max_scroll(computed: &ComputedNode) -> Vec2 {
    ((computed.content_size() - computed.size()) * computed.inverse_scale_factor()).max(Vec2::ZERO)
}

/// Where each finger's last press began and whether it became a pan.
///
/// An entry stays after the lift, until the same finger presses again, so the systems that
/// read the lift and the click (one frame later than the last drag) can still tell a pan from
/// a tap.
#[derive(Resource, Debug, Default)]
pub(crate) struct TouchPans {
    pans: HashMap<PointerId, Pan>,
    /// Presses that landed on a flinging area, as the node pressed and the area, whose
    /// pressed look is ended by [`end_caught_presses`] once the press has been delivered.
    caught: Vec<(Entity, Entity)>,
}

impl TouchPans {
    /// Whether the finger's last press moved past [`DRAG_THRESHOLD`] over a scroll area, so
    /// what it lifts on is not a tap: nothing is activated, picked or closed by it.
    pub(crate) fn panned(&self, pointer: PointerId) -> bool {
        self.pans
            .get(&pointer)
            .is_some_and(|pan| pan.panning || pan.caught)
    }
}

#[derive(Debug)]
struct Pan {
    /// The node the finger went down on.
    started_on: Entity,
    /// The innermost scrollable ancestor of `started_on`, or `None` when the finger went down
    /// on a control that is dragged itself (a slider, a stepper, a scroll bar) or on nothing
    /// that scrolls.
    target: Option<Entity>,
    /// How much of the finger's travel since the press has been scrolled.
    applied: Vec2,
    panning: bool,
    /// The finger went down on an area that was flinging and stopped it: like a pan, its lift
    /// is not a tap.
    caught: bool,
    /// The finger's travel since the press at the times it was seen (seconds of [`Time`]),
    /// the last ~[`VELOCITY_WINDOW`] of it, from which the fling starts.
    samples: VecDeque<(f32, Vec2)>,
}

/// Scrolls the innermost scroll area under a finger as the finger drags, once it has moved
/// past [`DRAG_THRESHOLD`], and ends the press the drag began on so the lift neither activates
/// nor picks it.
///
/// The area is routed as the wheel does: the nearest scrollable ancestor of the node the finger
/// went down on, stopping after a [`FixedNode`] (a popup that scrolls itself is found first),
/// and nothing is passed on at a limit. A finger that goes down on a [`Slider`], a
/// [`NumericStepper`] or a [`ScrollBar`] keeps its own drag and does not pan.
fn pan_with_finger(
    t: On<MouseEvent>,
    mut pans: ResMut<TouchPans>,
    hierarchy: Query<(Option<&ChildOf>, Has<FixedNode>)>,
    own_drag: Query<(), Or<(With<Slider>, With<NumericStepper>, With<ScrollBar>)>>,
    mut scrolled: Query<(&mut ScrollPosition, &ComputedNode, &Node)>,
    flinging: Query<(), With<Fling>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    let event = *t.event();
    let now = time.elapsed_secs();
    if !event.touch || event.button.is_some_and(|b| b != PointerButton::Primary) {
        return;
    }
    match event.kind {
        MouseKind::Pressed => {
            let target = find_pan_target(event.entity, &hierarchy, &own_drag, &scrolled);
            // A finger that lands on a flinging area stops it, and is not a tap on what it
            // landed on, as on iOS.
            let caught = target.is_some_and(|area| flinging.contains(area));
            if let Some(area) = target.filter(|_| caught) {
                commands.entity(area).try_remove::<Fling>();
                pans.caught.push((event.entity, area));
            }
            pans.pans.insert(
                event.pointer,
                Pan {
                    started_on: event.entity,
                    target,
                    applied: Vec2::ZERO,
                    panning: false,
                    caught,
                    samples: VecDeque::from([(now, Vec2::ZERO)]),
                },
            );
        }
        MouseKind::Drag => {
            let Some(pan) = pans.pans.get_mut(&event.pointer) else {
                return;
            };
            let Some(target) = pan.target else {
                return;
            };
            record_sample(&mut pan.samples, now, event.distance);
            if !pan.panning {
                if event.distance.length() <= DRAG_THRESHOLD {
                    return;
                }
                pan.panning = true;
                end_press(&mut commands, &hierarchy, pan.started_on, target);
            }
            let delta = event.distance - pan.applied;
            pan.applied = event.distance;
            let Ok((mut scroll_position, computed, node)) = scrolled.get_mut(target) else {
                return;
            };
            // A finger gives both axes, so an area that scrolls sideways alone takes a
            // mostly vertical drag as a horizontal one, as the wheel does.
            let sideways_only = node.overflow.y != OverflowAxis::Scroll;
            let dx = if sideways_only && delta.y.abs() > delta.x.abs() {
                delta.y
            } else {
                delta.x
            };
            scroll_by(&mut scroll_position, computed, node, dx, delta.y);
        }
        MouseKind::Released => {
            let Some(pan) = pans.pans.get_mut(&event.pointer) else {
                return;
            };
            let samples = std::mem::take(&mut pan.samples);
            let Some(area) = pan.target.filter(|_| pan.panning) else {
                return;
            };
            let Ok((scroll_position, _, node)) = scrolled.get(area) else {
                return;
            };
            let velocity = fling_velocity(release_velocity(&samples, now), node);
            if velocity.length() >= MIN_FLING_SPEED {
                commands.entity(area).try_insert(Fling {
                    velocity,
                    expected: scroll_position.0,
                });
            }
        }
        _ => {}
    }
}

/// How long a finger's last movement counts towards the speed it lets go with, in seconds.
const VELOCITY_WINDOW: f32 = 0.1;

/// The slowest release, in logical pixels a second, that still flings: a slower drag ends where
/// the finger lifted.
const MIN_FLING_SPEED: f32 = 100.0;

/// A fling ends when it is slower than this, in logical pixels a second: half a pixel in a frame
/// of a sixtieth of a second.
const STOP_FLING_SPEED: f32 = 30.0;

/// How much of its speed a fling keeps each millisecond: iOS `DecelerationRate.normal`.
const DECELERATION_PER_MS: f32 = 0.998;

/// Adds the finger's travel `distance` at time `now` to `samples`, forgetting what is older than
/// [`VELOCITY_WINDOW`] but the newest such sample, which the speed is measured from.
fn record_sample(samples: &mut VecDeque<(f32, Vec2)>, now: f32, distance: Vec2) {
    samples.push_back((now, distance));
    while samples
        .get(1)
        .is_some_and(|&(time, _)| time <= now - VELOCITY_WINDOW)
    {
        samples.pop_front();
    }
}

/// The finger's velocity in logical pixels a second at `now`, from the last [`VELOCITY_WINDOW`]
/// of `samples`: its travel since the newest sample at least that old (the oldest when none is),
/// over the time since. A finger that rested for the whole window has none.
fn release_velocity(samples: &VecDeque<(f32, Vec2)>, now: f32) -> Vec2 {
    let (Some(&(then, from)), Some(&(_, to))) = (
        samples
            .iter()
            .rev()
            .find(|&&(time, _)| time <= now - VELOCITY_WINDOW)
            .or_else(|| samples.front()),
        samples.back(),
    ) else {
        return Vec2::ZERO;
    };
    let elapsed = now - then;
    if elapsed <= f32::EPSILON {
        return Vec2::ZERO;
    }
    (to - from) / elapsed
}

/// The scroll velocity a finger moving at `finger` gives the area `node`: opposite to the finger,
/// along the axes the area scrolls, with a sideways-only area taking a mostly vertical finger as
/// horizontal, as a pan does.
fn fling_velocity(finger: Vec2, node: &Node) -> Vec2 {
    let scrolls_x = node.overflow.x == OverflowAxis::Scroll;
    let scrolls_y = node.overflow.y == OverflowAxis::Scroll;
    let x = if !scrolls_y && finger.y.abs() > finger.x.abs() {
        finger.y
    } else {
        finger.x
    };
    Vec2::new(
        if scrolls_x { -x } else { 0.0 },
        if scrolls_y { -finger.y } else { 0.0 },
    )
}

/// A scroll area that keeps scrolling after a finger let go of it, slowing down. Put on the area
/// when a fast pan ends and taken off when it stops, so only flinging areas are visited.
#[derive(Component, Debug)]
pub(crate) struct Fling {
    /// Scroll velocity in logical pixels a second.
    velocity: Vec2,
    /// Where the fling left the [`ScrollPosition`]. Anything else moving it (the wheel, a scroll
    /// bar, scrolling a row into view) shows as a difference, and ends the fling.
    expected: Vec2,
}

/// How far a speed of `velocity` (logical pixels a second) travels in `dt` seconds while it decays
/// by [`DECELERATION_PER_MS`] each millisecond, and what it is after. Integrated exactly, so the
/// distance does not depend on how the time is cut into frames.
fn decelerate(velocity: Vec2, dt: f32) -> (Vec2, Vec2) {
    // `v(t) = v0 * r^(1000 t)` for `r` per millisecond; its integral is `v0 (f - 1) / ln_rate`.
    let ln_rate = DECELERATION_PER_MS.ln() * 1000.0;
    let kept = (ln_rate * dt).exp();
    (velocity * ((kept - 1.0) / ln_rate), velocity * kept)
}

/// Moves each flinging area on by the time since the last frame, stopping an axis at its limit and
/// the fling when it is slow, or when something else moved the area.
///
/// Writes the [`ScrollPosition`] only when the result differs, so an area at its limit marks
/// nothing changed and does not close a combo in it.
pub(crate) fn step_flings(
    time: Res<Time>,
    mut flinging: Query<(
        Entity,
        &mut Fling,
        &mut ScrollPosition,
        &ComputedNode,
        &Node,
    )>,
    mut commands: Commands,
) {
    let dt = time.delta_secs();
    for (entity, mut fling, mut position, computed, node) in &mut flinging {
        if position.0 != fling.expected {
            commands.entity(entity).try_remove::<Fling>();
            continue;
        }
        let (distance, velocity) = decelerate(fling.velocity, dt);
        let max = max_scroll(computed);
        let wanted = position.0 + distance;
        let moved = wanted.clamp(Vec2::ZERO, max);
        let scrolls_x = node.overflow.x == OverflowAxis::Scroll;
        let scrolls_y = node.overflow.y == OverflowAxis::Scroll;
        // An axis that ran into its limit, or does not scroll, stops there.
        let velocity = Vec2::new(
            if scrolls_x && moved.x.to_bits() == wanted.x.to_bits() {
                velocity.x
            } else {
                0.0
            },
            if scrolls_y && moved.y.to_bits() == wanted.y.to_bits() {
                velocity.y
            } else {
                0.0
            },
        );
        let moved = Vec2::new(
            if scrolls_x { moved.x } else { position.x },
            if scrolls_y { moved.y } else { position.y },
        );
        if moved != position.0 {
            position.0 = moved;
        }
        if velocity.length() < STOP_FLING_SPEED {
            commands.entity(entity).try_remove::<Fling>();
        } else {
            fling.velocity = velocity;
            fling.expected = moved;
        }
    }
}

/// Ends the pressed look of the nodes a finger landed on while their area was flinging, so its
/// lift activates nothing. Runs after the frame's pointer events: the press that made the node
/// `Pressed` may be delivered after the observer that saw the touch.
fn end_caught_presses(
    mut pans: ResMut<TouchPans>,
    hierarchy: Query<(Option<&ChildOf>, Has<FixedNode>)>,
    mut commands: Commands,
) {
    for (started_on, area) in std::mem::take(&mut pans.caught) {
        end_press(&mut commands, &hierarchy, started_on, area);
    }
}

/// The scroll area a finger that went down on `entity` pans, if any.
fn find_pan_target(
    entity: Entity,
    hierarchy: &Query<(Option<&ChildOf>, Has<FixedNode>)>,
    own_drag: &Query<(), Or<(With<Slider>, With<NumericStepper>, With<ScrollBar>)>>,
    scrolled: &Query<(&mut ScrollPosition, &ComputedNode, &Node)>,
) -> Option<Entity> {
    let mut found = None;
    let mut current = Some(entity);
    while let Some(entity) = current {
        if own_drag.contains(entity) {
            return None;
        }
        if found.is_none()
            && scrolled.get(entity).is_ok_and(|(_, _, node)| {
                node.overflow.x == OverflowAxis::Scroll || node.overflow.y == OverflowAxis::Scroll
            })
        {
            found = Some(entity);
        }
        let (parent, fixed) = hierarchy.get(entity).ok()?;
        // Look on past the area for a control dragged itself only up to the area, and stop at
        // a floating popup, whose owner does not scroll with it.
        if found.is_some() || fixed {
            break;
        }
        current = parent.map(ChildOf::parent);
    }
    found
}

/// Ends the press a pan began, from the node the finger went down on up to the area: the
/// buttons among them lose `Pressed`, so the lift fires no `Activate` (Bevy's button needs it),
/// and a `Cancel` restores the resting look as a cancelled touch does.
fn end_press(
    commands: &mut Commands,
    hierarchy: &Query<(Option<&ChildOf>, Has<FixedNode>)>,
    started_on: Entity,
    area: Entity,
) {
    let mut entity = started_on;
    loop {
        commands.entity(entity).try_remove::<Pressed>();
        commands.trigger(MouseEvent {
            entity,
            kind: MouseKind::Cancel,
            distance: Vec2::ZERO,
            button: None,
            // Not marked as a finger's, so the pan does not take it for the finger going away.
            touch: false,
            pointer: PointerId::Mouse,
        });
        if entity == area {
            break;
        }
        let Ok((Some(parent), _)) = hierarchy.get(entity) else {
            break;
        };
        entity = parent.parent();
    }
}

/// Buffers [`update_scroll_position`] keeps between frames, so a wheel message allocates nothing.
#[derive(Default)]
pub(crate) struct WheelScratch {
    targets: HashSet<Entity>,
    outer_targets: HashSet<Entity>,
    resolved: Vec<Entity>,
}

/// Hides a scroll bar while the contents fit along its axis, and shows it again when
/// they overflow. The thumb's size and place are the [`Scrollbar`]'s. A bar is looked at when
/// its contents were laid out again or when its [`Scrollbar`] changed (a new orientation
/// means another axis).
fn show_scroll_bars(
    scroll_contents: Query<(Ref<ComputedNode>, &ScrollAreaRefs)>,
    mut scroll_bars: Query<(Ref<Scrollbar>, &mut Visibility), With<ScrollBar>>,
) {
    for (contents, refs) in scroll_contents {
        let axis = |orientation| match orientation {
            ControlOrientation::Horizontal => (contents.size().x, contents.content_size().x),
            ControlOrientation::Vertical => (contents.size().y, contents.content_size().y),
        };
        // Bevy's `Scrollbar` takes the visible length as `size() - scrollbar_size`; plain
        // `size()` agrees with it because `ScrollContents` sets no `scrollbar_width` (the
        // bars are nodes of their own beside it), so its `scrollbar_size` is zero.
        for bar in [refs.h_scrollbar, refs.v_scrollbar].into_iter().flatten() {
            let Ok((scrollbar, mut visibility)) = scroll_bars.get_mut(bar) else {
                continue;
            };
            if !contents.is_changed() && !scrollbar.is_changed() {
                continue;
            }
            let (visible_length, contents_length) = axis(scrollbar.orientation);
            if visible_length < f32::EPSILON {
                continue;
            }
            visibility.set_if_neq(if visible_length >= contents_length {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            });
        }
    }
}

fn theme_scroll_area(
    scroll_area_ref: In<Option<Entity>>,
    theme: Res<Theme>,
    mut scroll_contents: Query<(&mut BackgroundColor, &ScrollAreaRefs), With<ScrollContents>>,
    mut scroll_bars: Query<
        (
            &Scrollbar,
            &mut Node,
            &mut BackgroundColor,
            Option<&Children>,
        ),
        (With<ScrollBar>, Without<ScrollContents>),
    >,
    mut scroll_thumbs: Query<ThumbData, ThumbFilter>,
) {
    let it = if let Some(scroll_area_ref) = scroll_area_ref.as_ref() {
        Either::Left(scroll_contents.get_mut(*scroll_area_ref).into_iter())
    } else {
        Either::Right(scroll_contents.iter_mut())
    };

    for (mut background_color, refs) in it {
        background_color.0 = theme.colors().background_primary;
        for entity in [refs.v_scrollbar, refs.h_scrollbar].into_iter().flatten() {
            let Ok((scrollbar, mut sb_node, mut sb_background_color, children)) =
                scroll_bars.get_mut(entity)
            else {
                continue;
            };

            theme_bar(
                &theme,
                scrollbar,
                &mut sb_node,
                &mut sb_background_color,
                children,
                &mut scroll_thumbs,
            );
        }
    }
}

type ThumbData = (
    &'static mut ScrollbarThumb,
    &'static mut BackgroundColor,
    &'static mut BorderColor,
    &'static Hovered,
    Has<Pressed>,
    Option<&'static ScrollbarDragState>,
);
type ThumbFilter = (
    With<ScrollThumb>,
    Without<ScrollContents>,
    Without<ScrollBar>,
);

/// Themes one scroll bar and its marked thumb.
fn theme_bar(
    theme: &Theme,
    scrollbar: &Scrollbar,
    node: &mut Node,
    background: &mut BackgroundColor,
    children: Option<&Children>,
    thumbs: &mut Query<ThumbData, ThumbFilter>,
) {
    size_bar(node, scrollbar.orientation, theme.scroll_bar_width);
    background.0 = theme.colors().background_secondary;
    node.border_radius = BorderRadius::all(theme.control_radius);

    // Only the marked thumb is ours; other children of the bar are the app's.
    for thumb_entity in children.into_iter().flatten() {
        let Ok((mut thumb, mut thumb_color, mut thumb_border, hovered, pressed, drag)) =
            thumbs.get_mut(*thumb_entity)
        else {
            continue;
        };
        // The thumb is as thick as its scroll bar: the `Scrollbar` sizes it.
        thumb_color.0 = thumb_background(
            theme.colors(),
            hovered.get(),
            pressed || drag.is_some_and(|drag| drag.dragging),
        );
        thumb.border_radius = BorderRadius::all(theme.control_radius);
        thumb_border.set_all(theme.colors().border);
    }
}

/// The vertical scroll bar of a popup list that scrolls itself (a [`Combo`](crate::combo::Combo)'s
/// or [`FilterCombo`](crate::filter_combo::FilterCombo)'s list capped to the window).
///
/// It is a child of the list it scrolls and shows only while that list's contents overflow it.
/// The list scrolls, so a child placed at the top would scroll away with the rows:
/// `place_popup_scroll_bars` keeps the bar at the visible top of the list.
#[derive(Component, Clone, Copy, Debug)]
pub(crate) struct PopupScrollBar {
    list: Entity,
}

impl PopupScrollBar {
    /// The list this bar scrolls.
    pub(crate) const fn list(self) -> Entity {
        self.list
    }
}

/// The bundle of a [`PopupScrollBar`] for `list`, a child of it.
pub(crate) fn popup_scroll_bar(list: Entity) -> impl Bundle {
    (
        ScrollBar,
        Scrollbar::new(list, ControlOrientation::Vertical, 0.0),
        PopupScrollBar { list },
        Hovered::default(),
        Pickable::default(),
        Node {
            position_type: PositionType::Absolute,
            ..default()
        },
        Visibility::Hidden,
        ChildOf(list),
    )
}

/// Puts each popup bar at the right edge of the visible part of its list and shows it only
/// while the list's rows overflow.
fn place_popup_scroll_bars(
    lists: Query<(&ComputedNode, &ScrollPosition, &Node), Without<PopupScrollBar>>,
    mut bars: Query<(&PopupScrollBar, &mut Node, &mut Visibility)>,
) {
    for (bar, mut node, mut visibility) in &mut bars {
        let Ok((computed, position, list_node)) = lists.get(bar.list) else {
            continue;
        };
        let scale = computed.inverse_scale_factor();
        let visible = computed.size().y * scale;
        // Layout reports the content of a padded list (the separated one) as taller than its box
        // by the bottom padding even when every row fits (#641), so the padding is not overflow.
        let bottom_padding = match list_node.padding.bottom {
            Val::Px(px) => px.max(0.0),
            _ => 0.0,
        };
        let contents = computed.content_size().y.mul_add(scale, -bottom_padding);
        if visible < f32::EPSILON {
            continue;
        }
        visibility.set_if_neq(if visible >= contents {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        let top = Val::Px(position.0.y);
        let height = Val::Px(visible);
        let right = Val::Px(0.0);
        if node.top != top || node.height != height || node.right != right {
            node.top = top;
            node.height = height;
            node.right = right;
        }
    }
}

fn theme_popup_scroll_bars(
    bar: In<Option<Entity>>,
    theme: Res<Theme>,
    mut bars: Query<
        (
            &Scrollbar,
            &mut Node,
            &mut BackgroundColor,
            Option<&Children>,
        ),
        (
            With<PopupScrollBar>,
            With<ScrollBar>,
            Without<ScrollContents>,
        ),
    >,
    mut thumbs: Query<ThumbData, ThumbFilter>,
) {
    let it = if let Some(bar) = bar.0 {
        Either::Left(bars.get_mut(bar).into_iter())
    } else {
        Either::Right(bars.iter_mut())
    };
    for (scrollbar, mut node, mut background, children) in it {
        theme_bar(
            &theme,
            scrollbar,
            &mut node,
            &mut background,
            children,
            &mut thumbs,
        );
    }
}

/// Themes a new popup bar, and one that has just got its thumb.
fn theme_new_popup_scroll_bars(
    new: Query<
        Entity,
        (
            With<PopupScrollBar>,
            Or<(Added<PopupScrollBar>, Changed<Children>)>,
        ),
    >,
    mut commands: Commands,
) {
    for bar in &new {
        commands.run_system_cached_with(theme_popup_scroll_bars, Some(bar));
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests may abort loudly")]
mod tests {
    use super::*;

    fn samples(points: &[(f32, f32)]) -> VecDeque<(f32, Vec2)> {
        points
            .iter()
            .map(|&(t, y)| (t, Vec2::new(0.0, y)))
            .collect()
    }

    #[test]
    fn the_release_velocity_is_the_travel_over_the_last_tenth_of_a_second() {
        // 10 px every sixtieth of a second: 600 px a second, however long the pan was before.
        let mut recorded = VecDeque::new();
        for frame in 0..30_u8 {
            record_sample(
                &mut recorded,
                f32::from(frame) / 60.0,
                Vec2::new(0.0, -10.0 * f32::from(frame)),
            );
        }
        let velocity = release_velocity(&recorded, 29.0 / 60.0);
        assert!((velocity.y + 600.0).abs() < 1.0, "{velocity}");
        // Old samples are forgotten but for the one the speed is measured from.
        assert!(recorded.len() <= 8, "{}", recorded.len());
    }

    #[test]
    fn a_pan_that_sped_up_at_the_end_lets_go_at_the_end_speed() {
        // Slow for a second (1 px per frame), then 10 px per frame for the last tenth.
        let mut recorded = VecDeque::new();
        let mut y = 0.0;
        for frame in 0..60_u8 {
            y -= 1.0;
            record_sample(&mut recorded, f32::from(frame) / 60.0, Vec2::new(0.0, y));
        }
        for frame in 60..66_u8 {
            y -= 10.0;
            record_sample(&mut recorded, f32::from(frame) / 60.0, Vec2::new(0.0, y));
        }
        let velocity = release_velocity(&recorded, 65.0 / 60.0);
        assert!(velocity.y < -400.0, "{velocity}");
    }

    #[test]
    fn a_finger_that_rested_for_the_window_lets_go_at_no_speed() {
        let recorded = samples(&[(0.0, 0.0), (0.05, -40.0)]);
        assert_eq!(release_velocity(&recorded, 0.3), Vec2::ZERO);
        assert_eq!(release_velocity(&VecDeque::new(), 0.3), Vec2::ZERO);
        // No time between the sample and the release: no speed rather than a division by zero.
        assert_eq!(release_velocity(&samples(&[(0.2, 0.0)]), 0.2), Vec2::ZERO);
    }

    #[test]
    fn a_fling_goes_opposite_to_the_finger_along_the_axes_the_area_scrolls() {
        let vertical = Node {
            overflow: Overflow::scroll_y(),
            ..default()
        };
        let sideways = Node {
            overflow: Overflow::scroll_x(),
            ..default()
        };
        let finger = Vec2::new(30.0, -400.0);
        assert_eq!(fling_velocity(finger, &vertical), Vec2::new(0.0, 400.0));
        // A sideways-only area takes a mostly vertical finger as horizontal.
        assert_eq!(fling_velocity(finger, &sideways), Vec2::new(400.0, 0.0));
        assert_eq!(
            fling_velocity(
                finger,
                &Node {
                    overflow: Overflow::scroll(),
                    ..default()
                }
            ),
            Vec2::new(-30.0, 400.0)
        );
    }

    #[test]
    fn velocity_decays_by_0_998_a_millisecond_whatever_the_frame_length() {
        let (_, after) = decelerate(Vec2::new(1000.0, 0.0), 0.1);
        assert!(
            (after.x - 0.998_f32.powi(100).mul_add(1000.0, 0.0)).abs() < 0.01,
            "{after}"
        );
        let run = |frames: u32| {
            let dt = 10.0 / frames as f32;
            let (mut velocity, mut travelled) = (Vec2::new(1000.0, 0.0), 0.0);
            for _ in 0..frames {
                let (distance, next) = decelerate(velocity, dt);
                travelled += distance.x;
                velocity = next;
            }
            travelled
        };
        // About 1000 px a second over `1 / (1000 * -ln 0.998)` seconds in all.
        assert!((run(600) - 499.5).abs() < 2.0, "{}", run(600));
        assert!((run(600) - run(2400)).abs() < 0.1);
        assert!((run(600) - run(7)).abs() < 0.1);
    }

    #[test]
    fn a_fling_ends_on_stopping_and_an_idle_area_has_none_to_visit() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_secs_f32(1.0 / 60.0),
        ));
        app.add_systems(Update, step_flings.run_if(any_with_component::<Fling>));
        let area = app
            .world_mut()
            .spawn((
                Node {
                    overflow: Overflow::scroll_y(),
                    ..default()
                },
                ScrollPosition::default(),
                ComputedNode {
                    size: Vec2::splat(100.0),
                    content_size: Vec2::splat(10_000.0),
                    inverse_scale_factor: 1.0,
                    ..default()
                },
                Fling {
                    velocity: Vec2::new(0.0, 200.0),
                    expected: Vec2::ZERO,
                },
            ))
            .id();
        app.update();
        for _ in 0..600 {
            app.update();
        }
        assert!(!app.world().entity(area).contains::<Fling>());
        let rested = app.world().get::<ScrollPosition>(area).unwrap().0;
        assert!(rested.y > 50.0 && rested.x == 0.0, "{rested}");
        app.update();
        assert_eq!(app.world().get::<ScrollPosition>(area).unwrap().0, rested);
    }
}
