//! `ScrollArea`: scrollable contents with vertical and/or horizontal scroll bars.
//!
//! The scroll bars are `bevy_ui_widgets` [`Scrollbar`]s: the widget sizes and places
//! each thumb, scrolls the contents when the thumb is dragged and pages them on a click
//! on the track. This module keeps the layout, the look, the wheel and scrolling a
//! focused row into view.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::world::DeferredWorld;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui_widgets::{ControlOrientation, ScrollIntoView, Scrollbar, ScrollbarThumb};
use itertools::Either;

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
            .add_systems(PostUpdate, show_scroll_bars)
            .add_systems(
                Update,
                (|| None::<Entity>)
                    .pipe(theme_scroll_area)
                    .run_if(resource_changed::<Theme>),
            )
            .register_type::<ScrollBar>()
            .register_type::<ScrollAreaRefs>();
    }
}

/// A node with scroll bars. Build one with [`ScrollArea::bundle`].
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
                            // (taffy 0.10, Bevy 0.19), which left nothing to scroll; kept
                            // on taffy 0.14 until `Stretch` is retried.
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                    ))
                    .id();

                let mut spawn_bar = |orientation| {
                    s.spawn((
                        ScrollBar(orientation),
                        Scrollbar::new(contents_ref, orientation.control_orientation(), 0.0),
                    ))
                    .id()
                };
                let refs = ScrollAreaRefs {
                    v_scrollbar: v_scroll.then(|| spawn_bar(Orientation::Vertical)),
                    h_scrollbar: h_scroll.then(|| spawn_bar(Orientation::Horizontal)),
                };
                s.world_mut().entity_mut(contents_ref).insert(refs);
                s.world_mut()
                    .commands()
                    .run_system_cached_with(theme_scroll_area, Some(contents_ref));
            })),
        )
    }
}

/// A scroll bar with a thumb, along the given [`Orientation`].
///
/// [`ScrollArea::bundle`] spawns it with the `bevy_ui_widgets` [`Scrollbar`] that
/// scrolls the area's [`ScrollContents`].
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[require(Node::default(), Visibility::Inherited, Name::new("ScrollBar"))]
#[component(on_insert=Self::on_insert)]
pub struct ScrollBar(pub Orientation);

/// The axis of a [`ScrollBar`].
#[derive(Clone, Copy, Debug, Reflect)]
pub enum Orientation {
    /// Scrolls left and right.
    Horizontal,
    /// Scrolls up and down.
    Vertical,
}

impl Orientation {
    /// The same axis as `bevy_ui_widgets` names it.
    const fn control_orientation(self) -> ControlOrientation {
        match self {
            Self::Horizontal => ControlOrientation::Horizontal,
            Self::Vertical => ControlOrientation::Vertical,
        }
    }
}

impl ScrollBar {
    /// Component hook: places the scroll bar in the grid of its
    /// [`ScrollArea`] (the vertical bar right of the contents, the horizontal
    /// bar below them, whichever bars the area has) and spawns its thumb.
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let mut entity = world.entity_mut(context.entity);
        if let Ok((scroll_bar, mut node)) = entity.get_components_mut::<(&Self, &mut Node)>() {
            let (row, column) = match scroll_bar.0 {
                Orientation::Horizontal => (2, 1),
                Orientation::Vertical => (1, 2),
            };
            node.grid_row = GridPlacement::start(row);
            node.grid_column = GridPlacement::start(column);
        }
        world
            .commands()
            .entity(context.entity)
            .with_children(|spawner| {
                spawner.spawn(ScrollThumb).observe(handle_hover);
            });
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

/// Scrolls the [`ScrollContents`] around an entity just far enough to show it.
///
/// Answers [`ScrollIntoView`], which a `ListBox` triggers when the arrow keys
/// move its active descendant. An axis is moved only when the contents scroll
/// along it, and the [`ScrollPosition`] stays within `0..=max` as in
/// `update_scroll_position`. An entity outside any scroll area is left alone.
fn scroll_into_view(
    mut event: On<ScrollIntoView>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    parents: Query<&ChildOf>,
    mut contents: Query<(&mut ScrollPosition, &Node), With<ScrollContents>>,
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
    event.propagate(false);

    let target_scale = target_computed.inverse_scale_factor();
    let area_scale = area_computed.inverse_scale_factor();
    let target_size = target_computed.size() * target_scale;
    let area_size = area_computed.size() * area_scale;
    // Top left corners in logical pixels, as the layout last placed them (the
    // target's already shifted by the scroll position it had then).
    let target_top_left = target_transform.translation * target_scale - target_size * 0.5;
    let area_top_left = area_transform.translation * area_scale - area_size * 0.5;
    let local_top_left = target_top_left - area_top_left + scroll_position.0;
    let local_bottom_right = local_top_left + target_size;
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
    mut scroll_bars: Query<&mut Visibility, With<ScrollBar>>,
) {
    for (contents, refs) in scroll_contents {
        let bars = [
            (
                refs.h_scrollbar,
                contents.size().x,
                contents.content_size().x,
            ),
            (
                refs.v_scrollbar,
                contents.size().y,
                contents.content_size().y,
            ),
        ];
        for (bar, visible_length, contents_length) in bars {
            let Some(mut visibility) = bar.and_then(|bar| scroll_bars.get_mut(bar).ok()) else {
                continue;
            };
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
        (&mut Node, &mut BackgroundColor, &Children),
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
        let mut theme_scrollbar = |entity, orientation| {
            let Ok((mut sb_node, mut sb_background_color, children)) = scroll_bars.get_mut(entity)
            else {
                return;
            };

            match orientation {
                Orientation::Horizontal => {
                    sb_node.height = theme.scroll_bar_width;
                }
                Orientation::Vertical => {
                    sb_node.width = theme.scroll_bar_width;
                }
            }
            sb_background_color.0 = theme.colors.background_secondary;
            sb_node.border_radius = BorderRadius::all(theme.control_radius);

            let Some((mut thumb, mut thumb_background_color, mut thumb_border_color)) = children
                .iter()
                .next()
                .and_then(|thumb_entity| scroll_thumbs.get_mut(thumb_entity).ok())
            else {
                return;
            };

            // The thumb is as thick as its scroll bar: the `Scrollbar` sizes it.
            thumb_background_color.0 = theme.colors.scroll_thumb_background;
            thumb.border_radius = BorderRadius::all(theme.control_radius);
            thumb_border_color.set_all(theme.colors.border);
        };

        if let Some(x) = refs.v_scrollbar {
            theme_scrollbar(x, Orientation::Vertical);
        }
        if let Some(x) = refs.h_scrollbar {
            theme_scrollbar(x, Orientation::Horizontal);
        }
    }
}
