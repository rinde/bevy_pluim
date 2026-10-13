//! `ScrollArea`: scrollable contents with vertical and/or horizontal scroll bars.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::world::DeferredWorld;
use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::hover::HoverMap;
use bevy::prelude::*;
use bevy::ui_widgets::ScrollIntoView;
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
            .add_systems(PostUpdate, handle_scroll_change)
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
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
#[require(Node::default(), Name::new("ScrollArea"))]
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
                            // this node's height reports no overflow when it is a grid
                            // (taffy 0.10), which would leave nothing to scroll.
                            align_items: AlignItems::FlexStart,
                            ..default()
                        },
                    ))
                    .id();

                let refs = ScrollAreaRefs {
                    v_scrollbar: v_scroll.then(|| s.spawn(ScrollBar(Orientation::Vertical)).id()),
                    h_scrollbar: h_scroll.then(|| s.spawn(ScrollBar(Orientation::Horizontal)).id()),
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
                spawner
                    .spawn(ScrollThumb)
                    .observe(handle_hover)
                    .observe(handle_thumb_drag);
            });
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(
    Button,
    Node {
        position_type: PositionType::Absolute,
        ..default()
    },
    Name::new("ScrollThumb")
)]
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
        MouseKind::Click(_) | MouseKind::Drag(_) => return,
    };
    if let Ok(mut background) = query.get_mut(trigger.event().event_target()) {
        *background = BackgroundColor(color);
    }
}

/// Scrolls the contents by as much as the thumb moved along its scroll bar.
fn handle_thumb_drag(
    drag: On<MouseEvent>,
    thumbs: Query<&ChildOf, With<ScrollThumb>>,
    scroll_bars: Query<&ScrollBar>,
    mut scroll_contents: Query<(&mut ScrollPosition, &ComputedNode, &ScrollAreaRefs)>,
) {
    let MouseKind::Drag(delta) = drag.event().kind else {
        return;
    };
    let Ok(scroll_bar_entity) = thumbs.get(drag.event().event_target()).map(ChildOf::parent) else {
        return;
    };
    let Ok(scroll_bar) = scroll_bars.get(scroll_bar_entity) else {
        return;
    };
    let (index, is_this_bar): (usize, fn(&ScrollAreaRefs) -> Option<Entity>) = match scroll_bar.0 {
        Orientation::Horizontal => (0, |refs| refs.h_scrollbar),
        Orientation::Vertical => (1, |refs| refs.v_scrollbar),
    };
    let Some((mut scroll_position, contents_dims, _)) = scroll_contents
        .iter_mut()
        .find(|(_, _, refs)| is_this_bar(refs) == Some(scroll_bar_entity))
    else {
        return;
    };

    let contents_length =
        contents_dims.content_size()[index] * contents_dims.inverse_scale_factor();
    let visible_length = contents_dims.size()[index] * contents_dims.inverse_scale_factor();
    let max_scroll_offset = contents_length - visible_length;
    // The thumb is as long as the visible part is of the contents (see
    // `scroll_system`), so it travels the rest of the visible length.
    let track_length = visible_length * (1.0 - visible_length / contents_length);
    if max_scroll_offset < f32::EPSILON || track_length < f32::EPSILON {
        return;
    }
    // Start from the current `ScrollPosition`, not from where the layout last
    // showed the contents (`ComputedNode::scroll_position`): that is floored
    // and only refreshed by the next layout pass, so the several drags a
    // frame can hold would each start from the same value and only the last
    // would count. The wheel keeps `ScrollPosition` in range (see
    // `update_scroll_position`), so the clamp is only a guard, for a content
    // size that shrank since.
    let start = scroll_position.0[index].clamp(0.0, max_scroll_offset);
    scroll_position.0[index] =
        (start + delta[index] * max_scroll_offset / track_length).clamp(0.0, max_scroll_offset);
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

fn handle_scroll_change(
    scroll_contents: Query<
        Entity,
        (
            With<ScrollContents>,
            Or<(Changed<ScrollPosition>, Changed<ComputedNode>)>,
        ),
    >,
    mut commands: Commands,
) {
    for e in scroll_contents {
        commands.run_system_cached_with(scroll_system, e);
    }
}

fn scroll_system(
    changed_scroll_contents_entity: In<Entity>,
    scroll_contents: Query<(&ScrollPosition, &ComputedNode, &ScrollAreaRefs), With<ScrollContents>>,
    mut scroll_bars: Query<(&ComputedNode, &Children, &mut Visibility), With<ScrollBar>>,
    mut scroll_thumbs: Query<&mut Node, With<ScrollThumb>>,
) {
    let Ok((scroll_pos, contents_dims, refs)) =
        scroll_contents.get(*changed_scroll_contents_entity)
    else {
        return;
    };

    let mut update_thumb = |entity: Entity, orientation| {
        let Ok((_, scrollbar_children, mut scrollbar_visibility)) = scroll_bars.get_mut(entity)
        else {
            return;
        };
        let Some(mut thumb_node) = scrollbar_children
            .iter()
            .next()
            .and_then(|thumb| scroll_thumbs.get_mut(thumb).ok())
        else {
            return;
        };

        let index = match orientation {
            Orientation::Horizontal => 0,
            Orientation::Vertical => 1,
        };
        let contents_length =
            contents_dims.content_size()[index] * contents_dims.inverse_scale_factor();
        let visible_length = contents_dims.size()[index] * contents_dims.inverse_scale_factor();

        let mut thumb_length = if visible_length < f32::EPSILON {
            0.0
        } else if visible_length >= contents_length {
            *scrollbar_visibility = Visibility::Hidden;
            visible_length
        } else {
            *scrollbar_visibility = Visibility::Inherited;
            (visible_length / contents_length) * visible_length
        };

        let max_scroll_offset = contents_length - visible_length;
        let mut ratio = if max_scroll_offset < f32::EPSILON {
            0.0
        } else {
            scroll_pos.0[index] / max_scroll_offset
        };

        // create scrollbar bouncing/shrinking effect when reaching the edges
        if ratio < 0.0 {
            thumb_length *= (1.0 + ratio).clamp(0.7, 1.0);
            ratio = 0.0;
        } else if ratio > 1.0 {
            thumb_length *= (2.0 - ratio).clamp(0.7, 1.0);
            ratio = 1.0;
        }

        match orientation {
            Orientation::Horizontal => {
                thumb_node.width = Val::Px(thumb_length);
                thumb_node.left =
                    Val::Px((ratio * (visible_length - thumb_length)).clamp(0.0, visible_length));
            }
            Orientation::Vertical => {
                thumb_node.height = Val::Px(thumb_length);
                thumb_node.top =
                    Val::Px((ratio * (visible_length - thumb_length)).clamp(0.0, visible_length));
            }
        }
    };

    if let Some(v_scroll) = refs.v_scrollbar {
        update_thumb(v_scroll, Orientation::Vertical);
    }
    if let Some(h_scroll) = refs.h_scrollbar {
        update_thumb(h_scroll, Orientation::Horizontal);
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
        (&mut Node, &mut BackgroundColor, &mut BorderColor),
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

            let Some((mut thumb_node, mut thumb_background_color, mut thumb_border_color)) =
                children
                    .iter()
                    .next()
                    .and_then(|thumb_entity| scroll_thumbs.get_mut(thumb_entity).ok())
            else {
                return;
            };

            match orientation {
                Orientation::Horizontal => {
                    thumb_node.height = theme.scroll_bar_width;
                }
                Orientation::Vertical => {
                    thumb_node.width = theme.scroll_bar_width;
                }
            }
            // thumb_node.border = UiRect::all(theme.border_thickness);
            thumb_background_color.0 = theme.colors.scroll_thumb_background;
            thumb_node.border_radius = BorderRadius::all(theme.control_radius);
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
