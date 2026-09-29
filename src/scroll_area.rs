use bevy::ecs::component::HookContext;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::MouseEvent;
use crate::theme::Theme;

#[derive(Copy, Clone, Debug)]
pub struct ScrollAreaPlugin;

impl Plugin for ScrollAreaPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, handle_scroll_change)
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

#[derive(Component, Clone, Copy, Debug)]
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

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollContents;

#[derive(Component, Clone, Copy, Debug, Reflect)]
struct ScrollAreaRefs {
    v_scrollbar: Option<Entity>,
    h_scrollbar: Option<Entity>,
}

impl ScrollArea {
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

#[derive(Component, Clone, Copy, Debug, Reflect)]
#[require(Node::default(), Visibility::Inherited, Name::new("ScrollBar"))]
#[component(on_insert=Self::on_insert)]
pub struct ScrollBar(pub Orientation);

#[derive(Clone, Copy, Debug, Reflect)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl ScrollBar {
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .entity(context.entity)
            .with_children(|spawner| {
                spawner.spawn(ScrollThumb).observe(handle_hover);
            });
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(
    Button,
    Node {
        position_type: PositionType::Absolute,
        ..default()
    }
)]
struct ScrollThumb;

// TODO wait for 0.17 to implement dragging thumb
fn handle_hover(
    trigger: Trigger<MouseEvent>,
    mut query: Query<&mut BackgroundColor>,
    theme: Res<Theme>,
) {
    let color = match trigger.event() {
        MouseEvent::Over => theme.colors.hovered_item,
        MouseEvent::Pressed => theme.colors.selected_hovered_item,
        MouseEvent::Released => theme.colors.selected_item,
        MouseEvent::Out => theme.colors.scroll_thumb_background,
    };
    if let Ok(mut background) = query.get_mut(trigger.target()) {
        *background = BackgroundColor(color);
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
            Vec2::from(scroll_pos)[index] / max_scroll_offset
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
        (
            &mut Node,
            &mut BackgroundColor,
            &mut BorderRadius,
            &Children,
        ),
        (With<ScrollBar>, Without<ScrollContents>),
    >,
    mut scroll_thumbs: Query<
        (
            &mut Node,
            &mut BackgroundColor,
            &mut BorderRadius,
            &mut BorderColor,
        ),
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
            let Ok((mut sb_node, mut sb_background_color, mut sb_border_radius, children)) =
                scroll_bars.get_mut(entity)
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
            *sb_border_radius = BorderRadius::all(theme.corner_radius);

            let Some((
                mut thumb_node,
                mut thumb_background_color,
                mut thumb_border_radius,
                mut thumb_border_color,
            )) = children
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
            *thumb_border_radius = BorderRadius::all(theme.corner_radius);
            thumb_border_color.0 = theme.colors.border;
        };

        if let Some(x) = refs.v_scrollbar {
            theme_scrollbar(x, Orientation::Vertical);
        }
        if let Some(x) = refs.h_scrollbar {
            theme_scrollbar(x, Orientation::Horizontal);
        }
    }
}
