use bevy::app::Plugin;
use bevy::app::PostUpdate;
use bevy::app::Update;
use bevy::color::Color;
use bevy::color::palettes::tailwind::GRAY_200;
use bevy::color::palettes::tailwind::RED_400;
use bevy::color::palettes::tailwind::TEAL_400;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::query::Changed;
use bevy::ecs::query::Or;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::Commands;
use bevy::ecs::system::In;
use bevy::ecs::system::IntoSystem;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::math::Vec2;
use bevy::prelude::SpawnRelated;
use bevy::render::view::Visibility;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::ComputedNode;
use bevy::ui::Display;
use bevy::ui::FlexDirection;
use bevy::ui::FlexWrap;
use bevy::ui::JustifyContent;
use bevy::ui::Node;
use bevy::ui::Overflow;
use bevy::ui::PositionType;
use bevy::ui::RepeatedGridTrack;
use bevy::ui::ScrollPosition;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::utils::default;
use educe::Educe;
use itertools::Either;

use crate::theme::Theme;

#[derive(Copy, Clone, Debug)]
pub struct ScrollAreaPlugin;

impl Plugin for ScrollAreaPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(PostUpdate, handle_scroll_change)
            .add_systems(
                Update,
                (|| None::<Entity>)
                    .pipe(theme_scroll_area)
                    .run_if(resource_changed::<Theme>),
            );
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node {
    // ensure these options are always set
    // display: Display::Grid,
    // grid_template_columns: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
    // grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
    // allow below to change
    width: Val::Px(400.0),
    height: Val::Px(600.0),
    row_gap: Val::Px(2.0),
    column_gap: Val::Px(2.0),
    ..default()
})]
#[component(on_insert=Self::on_insert)]
pub struct ScrollArea {
    v_scroll: bool,
    h_scroll: bool,
}

impl ScrollArea {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        // world.commands().ru

        let mut entity = world.entity_mut(context.entity);
        let mut node = entity.get_mut::<Node>().unwrap();

        node.display = Display::Grid;
        node.grid_template_columns =
            vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)];
        node.grid_template_rows = vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)];

        // world
        //     .commands()
        //     .run_system_cached_with(scroll_system, context.entity);

        // let refs = *world
        //     .entity(context.entity)
        //     .get::<ScrollAreaRefs>()
        //     .unwrap();

        // world.commands().entity(refs.contents).insert(refs);
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node {
    // display: Display::Flex,
    // flex_direction: FlexDirection::Row,
    // flex_wrap: FlexWrap::Wrap,
    // justify_content: JustifyContent::SpaceBetween,
    // align_content: bevy::ui::AlignContent::FlexStart,
    // row_gap: Val::Px(2.0),
    // column_gap: Val::Px(2.0),
    // padding: UiRect::all(Val::Px(4.0)),
    width : Val::Percent(100.0),
    height: Val::Percent(100.0),
    overflow: Overflow::scroll(),
    ..default()
})]
#[component(on_insert=Self::on_insert)]
pub struct ScrollContents {}

impl ScrollContents {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        // world.commands().ru

        // world
        //     .commands()
        //     .run_system_cached_with(scroll_system, context.entity);

        // let refs = *world
        //     .entity(context.entity)
        //     .get::<ScrollAreaRefs>()
        //     .unwrap();

        // world.commands().entity(refs.contents).insert(refs);
    }
}

#[derive(Component, Clone, Copy, Debug)]
struct ScrollAreaRefs {
    v_scrollbar: Option<Entity>,
    h_scrollbar: Option<Entity>,
}

impl ScrollArea {
    pub fn new<F>(contents_spawner: F, v_scroll: bool, h_scroll: bool) -> impl Bundle
    where
        F: FnOnce(&mut RelatedSpawner<ChildOf>) + Send + Sync + 'static,
    {
        let bundle = (
            ScrollArea { v_scroll, h_scroll },
            Children::spawn(SpawnWith(move |s: &mut RelatedSpawner<ChildOf>| {
                let contents_ref = s
                    .spawn((
                        ScrollContents {},
                        Children::spawn(SpawnWith(contents_spawner)),
                    ))
                    .id();

                let refs = ScrollAreaRefs {
                    v_scrollbar: v_scroll.then(|| s.spawn(ScrollBar(Orientation::Vertical)).id()),
                    h_scrollbar: h_scroll.then(|| s.spawn(ScrollBar(Orientation::Horizontal)).id()),
                };
                s.world_mut().entity_mut(contents_ref).insert(refs);
            })),
        );
        bundle
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node::default())]
#[component(on_insert=Self::on_insert)]
pub struct ScrollBar(pub Orientation);

#[derive(Clone, Copy, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl ScrollBar {
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        // let x = world.get::<ScrollBar>(context.entity).unwrap();

        // let node = match x.0 {
        //     Orientation::Horizontal => Node {
        //         min_height: Val::Px(8.0),
        //         ..default()
        //     },
        //     Orientation::Vertical => Node {
        //         min_width: Val::Px(8.0),
        //         ..default()
        //     },
        // };

        // world.commands().entity(context.entity).insert(node);

        world
            .commands()
            .entity(context.entity)
            .with_children(|spawner| {
                //
                spawner.spawn(ScrollThumb);
            });
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node{
    position_type: PositionType::Absolute,
..default()
})]
struct ScrollThumb;

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
    println!(
        "changed_scroll_contents_entity {:?}",
        changed_scroll_contents_entity
    );
    let (scroll_pos, contents_dims, refs) = scroll_contents
        .get(*changed_scroll_contents_entity)
        .unwrap();

    let mut update_thumb = |entity: Entity, orientation| {
        let (_, scrollbar_children, mut scrollbar_visibility) =
            scroll_bars.get_mut(entity).unwrap();
        let thumb = scrollbar_children.iter().next().unwrap();
        let mut thumb_node = scroll_thumbs.get_mut(*thumb).unwrap();

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
            *scrollbar_visibility = Visibility::Visible;
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
                (*thumb_node).left =
                    Val::Px((ratio * (visible_length - thumb_length)).clamp(0.0, visible_length));
            }
            Orientation::Vertical => {
                thumb_node.height = Val::Px(thumb_length);
                (*thumb_node).top =
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
        Either::Left([scroll_contents.get_mut(*scroll_area_ref).unwrap()].into_iter())
    } else {
        Either::Right(scroll_contents.iter_mut())
    };

    for (mut background_color, refs) in it {
        background_color.0 = theme.colors.background_primary;
        let mut theme_scrollbar = |entity, orientation| {
            let (mut sb_node, mut sb_background_color, mut sb_border_radius, children) =
                scroll_bars.get_mut(entity).unwrap();

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

            let thumb_entity = children.iter().next().unwrap();
            let (
                mut thumb_node,
                mut thumb_background_color,
                mut thumb_border_radius,
                mut thumb_border_color,
            ) = scroll_thumbs.get_mut(*thumb_entity).unwrap();

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
