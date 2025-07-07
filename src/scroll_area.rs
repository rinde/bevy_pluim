use bevy::app::Plugin;
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
use bevy::ecs::query::With;
use bevy::ecs::relationship::RelatedSpawner;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::Query;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::SpawnRelated;
use bevy::ui::BackgroundColor;
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

#[derive(Copy, Clone, Debug)]
pub struct ScrollAreaPlugin;

impl Plugin for ScrollAreaPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, scroll_system);
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node {
    display: Display::Grid,
    width: Val::Px(400.0),
    height: Val::Px(600.0),
    grid_template_columns: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
    grid_template_rows: vec![RepeatedGridTrack::flex(1, 1.), RepeatedGridTrack::auto(1)],
    row_gap: Val::Px(2.0),
    column_gap: Val::Px(2.0),
    ..default()
})]
// #[component(on_insert=Self::on_insert)]
pub struct ScrollArea {
    v_scroll: bool,
    h_scroll: bool,
}

impl ScrollArea {
    // fn on_insert(mut world: DeferredWorld, context: HookContext) {
    //     let refs = *world
    //         .entity(context.entity)
    //         .get::<ScrollAreaRefs>()
    //         .unwrap();

    //     world.commands().entity(refs.contents).insert(refs);
    // }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node {
    display: Display::Flex,
    flex_direction: FlexDirection::Row,
    flex_wrap: FlexWrap::Wrap,
    justify_content: JustifyContent::SpaceBetween,
    align_content: bevy::ui::AlignContent::FlexStart,
    row_gap: Val::Px(2.0),
    column_gap: Val::Px(2.0),
    padding: UiRect::all(Val::Px(4.0)),
    width : Val::Percent(100.0),
    height: Val::Percent(100.0),
    overflow: Overflow::scroll(),
    ..default()
}, BackgroundColor(Color::Srgba(GRAY_200)))]
pub struct ScrollContents {}

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
        (bundle)
    }
}

#[derive(Component, Clone, Copy, Debug)]
#[require(Node::default(), BackgroundColor(Color::Srgba(TEAL_400)))]
#[component(on_insert=Self::on_insert)]
pub struct ScrollBar(pub Orientation);

#[derive(Clone, Copy, Debug)]
pub enum Orientation {
    Horizontal,
    Vertical,
}

impl ScrollBar {
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let x = world.get::<ScrollBar>(context.entity).unwrap();

        let node = match x.0 {
            Orientation::Horizontal => Node {
                min_height: Val::Px(8.0),
                ..default()
            },
            Orientation::Vertical => Node {
                min_width: Val::Px(8.0),
                ..default()
            },
        };

        world.commands().entity(context.entity).insert(node);

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
    width: Val::Px(20.0),
    height: Val::Px(20.0),
..default()
}, BackgroundColor(Color::Srgba(RED_400)))]
struct ScrollThumb;

fn scroll_system(
    scroll_areas: Query<
        (Entity, &ScrollPosition, &ComputedNode, &ScrollAreaRefs),
        (With<ScrollContents>, Changed<ScrollPosition>),
    >,
    scroll_bars: Query<(&ComputedNode, &Children), With<ScrollBar>>,
    mut scroll_thumbs: Query<(&mut Node, &ComputedNode), With<ScrollThumb>>,
    // mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    // TODO change size of thumb
    for (e, sp, area_dims, refs) in scroll_areas {
        println!("scroll area {e}");
        // let sp = scrolled_node_query.get_mut(e).unwrap();
        println!("{sp:?}");

        let (_, scrollbar_children) = scroll_bars.get(refs.v_scrollbar.unwrap()).unwrap();
        let thumb = scrollbar_children.iter().next().unwrap();
        let (mut thumb_node, thumb_dim) = scroll_thumbs.get_mut(*thumb).unwrap();

        let contents_height = area_dims.content_size().y * area_dims.inverse_scale_factor();
        let visible_height = area_dims.size().y * area_dims.inverse_scale_factor();
        // let scrollbar_height = scrollbar_dim.size().y * scrollbar_dim.inverse_scale_factor();
        let thumb_height = thumb_dim.size().y * thumb_dim.inverse_scale_factor();

        println!(
            "contents_height: {contents_height}, visible_height: {visible_height}, thumb_height: {thumb_height}"
        );
        let max_scroll_offset = contents_height - visible_height;
        let ratio = if sp.offset_y < f32::EPSILON {
            0.0
        } else {
            sp.offset_y / max_scroll_offset
        };
        println!("ratio: {ratio} max_scroll_offset: {max_scroll_offset}");

        // let max_offset_y = contents_height - scrollbar_height;
        // let ratio =
        // let thumb_offset_y =

        (*thumb_node).top =
            Val::Px((ratio * (visible_height - thumb_height)).clamp(0.0, visible_height));
    }
}
