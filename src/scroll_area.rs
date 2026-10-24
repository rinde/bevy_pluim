//! `ScrollArea`: scrollable contents with vertical and/or horizontal scroll bars.
//!
//! The scroll bars are `bevy_ui_widgets` [`Scrollbar`]s: the widget sizes and places
//! each thumb, scrolls the contents when the thumb is dragged and pages them on a click
//! on the track. This module keeps the layout, the look, the wheel and scrolling a
//! focused row into view.

use std::sync::{Arc, Mutex};

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::template::TemplateContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::scene::EmptySceneEffect;
use bevy::ui::{FixedNode, UiSystems};
use bevy::ui_widgets::{ControlOrientation, ScrollIntoView, Scrollbar, ScrollbarThumb};
use itertools::Either;

use crate::focus::FOCUS_RING_REACH;
use crate::theme::Theme;
use crate::{MouseEvent, MouseKind};

/// Registers the systems and observers of [`ScrollArea`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct ScrollAreaPlugin;

impl Plugin for ScrollAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(scroll_into_view)
            .add_systems(Update, update_scroll_position)
            .add_systems(
                PostUpdate,
                (
                    refresh_changed_scroll_bars.before(UiSystems::Layout),
                    show_scroll_bars,
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

    /// A scroll area for a `bsn!` scene, whose contents are the scene list `contents`.
    ///
    /// Use it as an entry of a scene (`@scroll_area(..)` of a function returning it, or by
    /// composing it into another scene). `v_scroll` and `h_scroll` enable scrolling (and a
    /// scroll bar) on that axis. It builds the same area as [`ScrollArea::bundle`], which stays
    /// for code that spawns through `Commands`.
    pub fn scene(contents: impl SceneList, v_scroll: bool, h_scroll: bool) -> impl Scene {
        // A scene's template must be `Clone`, a scene list is not: the first build takes it.
        let contents = Arc::new(Mutex::new(Some(contents)));
        bsn! {
            template(move |context| {
                Ok(insert_after_scene(context, Self::bundle(
                    {
                        let contents = contents.lock().ok().and_then(|mut taken| taken.take());
                        move |parent: &mut ChildSpawner| {
                            let Some(contents) = contents else {
                                error!("the contents of a scroll area scene were already spawned");
                                return;
                            };
                            let target = parent.target_entity();
                            parent
                                .world_mut()
                                .entity_mut(target)
                                .queue_spawn_related_scenes::<Children>(contents);
                        }
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
#[require(ScrollbarThumb, Pickable::default(), Name::new("ScrollThumb"))]
struct ScrollThumb;

fn handle_hover(
    trigger: On<MouseEvent>,
    mut query: Query<&mut BackgroundColor>,
    theme: Res<Theme>,
) {
    let color = match trigger.event().kind {
        MouseKind::Over => theme.colors.hovered_item,
        MouseKind::Pressed => theme.colors.selected_hovered_item,
        MouseKind::Released => theme.colors.selected_item,
        MouseKind::Out => theme.colors.scroll_thumb_background,
        // A click or a drag leaves the colour of the press and release before it.
        MouseKind::Click(_) | MouseKind::Drag => return,
    };
    if let Ok(mut background) = query.get_mut(trigger.event().event_target()) {
        *background = BackgroundColor(color);
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
/// Nested areas: only nodes in picking's [`HoverMap`] are scrolled, and a
/// [`ScrollContents`] blocks the pointer for what is below it, so over an inner area
/// only that one scrolls; the area around it gets nothing, also when the inner one is at
/// its limit.
///
/// Registered once, by [`ScrollAreaPlugin`].
fn update_scroll_position(
    mut mouse_wheel_events: MessageReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<(&mut ScrollPosition, &ComputedNode, &Node)>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    for mouse_wheel_event in mouse_wheel_events.read() {
        let (mut dx, mut dy) = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => (mouse_wheel_event.x * 21., mouse_wheel_event.y * 21.),
            MouseScrollUnit::Pixel => (mouse_wheel_event.x, mouse_wheel_event.y),
        };

        if keyboard_input.pressed(KeyCode::ControlLeft)
            || keyboard_input.pressed(KeyCode::ControlRight)
        {
            std::mem::swap(&mut dx, &mut dy);
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map {
                let Ok((mut scroll_position, computed, node)) =
                    scrolled_node_query.get_mut(*entity)
                else {
                    continue;
                };
                let overflow =
                    (computed.content_size() - computed.size()) * computed.inverse_scale_factor();
                // A mouse wheel only turns vertically, so an area that scrolls sideways
                // alone takes the vertical turn for a horizontal one.
                let dx = if dx == 0.0 && node.overflow.y != OverflowAxis::Scroll {
                    dy
                } else {
                    dx
                };
                if node.overflow.x == OverflowAxis::Scroll {
                    scroll_position.x = (scroll_position.x - dx).clamp(0.0, overflow.x.max(0.0));
                }
                if node.overflow.y == OverflowAxis::Scroll {
                    scroll_position.y = (scroll_position.y - dy).clamp(0.0, overflow.y.max(0.0));
                }
            }
        }
    }
}

/// Hides a scroll bar while the contents fit along its axis, and shows it again when
/// they overflow. The thumb's size and place are the [`Scrollbar`]'s.
fn show_scroll_bars(
    scroll_contents: Query<(&ComputedNode, &ScrollAreaRefs), Changed<ComputedNode>>,
    mut scroll_bars: Query<(&Scrollbar, &mut Visibility), With<ScrollBar>>,
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
    mut scroll_thumbs: Query<
        (&mut ScrollbarThumb, &mut BackgroundColor, &mut BorderColor),
        (
            With<ScrollThumb>,
            Without<ScrollContents>,
            Without<ScrollBar>,
        ),
    >,
) {
    let it = if let Some(scroll_area_ref) = scroll_area_ref.as_ref() {
        Either::Left(scroll_contents.get_mut(*scroll_area_ref).into_iter())
    } else {
        Either::Right(scroll_contents.iter_mut())
    };

    for (mut background_color, refs) in it {
        background_color.0 = theme.colors.background_primary;
        for entity in [refs.v_scrollbar, refs.h_scrollbar].into_iter().flatten() {
            let Ok((scrollbar, mut sb_node, mut sb_background_color, children)) =
                scroll_bars.get_mut(entity)
            else {
                continue;
            };

            size_bar(&mut sb_node, scrollbar.orientation, theme.scroll_bar_width);
            sb_background_color.0 = theme.colors.background_secondary;
            sb_node.border_radius = BorderRadius::all(theme.control_radius);

            // Only the marked thumb is ours; other children of the bar are the app's.
            for thumb_entity in children.into_iter().flatten() {
                let Ok((mut thumb, mut thumb_background_color, mut thumb_border_color)) =
                    scroll_thumbs.get_mut(*thumb_entity)
                else {
                    continue;
                };
                // The thumb is as thick as its scroll bar: the `Scrollbar` sizes it.
                thumb_background_color.0 = theme.colors.scroll_thumb_background;
                thumb.border_radius = BorderRadius::all(theme.control_radius);
                thumb_border_color.set_all(theme.colors.border);
            }
        }
    }
}
