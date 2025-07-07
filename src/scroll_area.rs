use bevy::app::Plugin;
use bevy::app::Update;
use bevy::color::Color;
use bevy::color::palettes::tailwind::GRAY_200;
use bevy::color::palettes::tailwind::RED_400;
use bevy::color::palettes::tailwind::TEAL_400;
use bevy::ecs::bundle::Bundle;
use bevy::ecs::children;
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

#[derive(Copy, Clone, Debug)]
pub struct ScrollAreaPlugin;

impl Plugin for ScrollAreaPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, scroll_system);
    }
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollArea2 {
    v_scroll: bool,
    h_scroll: bool,
}

#[derive(Component, Clone, Copy, Debug)]
pub struct ScrollContents2 {}

impl ScrollArea2 {
    pub fn new<F>(children_spawner: F, v_scroll: bool, h_scroll: bool) -> impl Bundle
    where
        F: FnOnce(&mut RelatedSpawner<ChildOf>) + Send + Sync + 'static,
    {
        (
            ScrollArea2 { v_scroll, h_scroll },
            children![(
                ScrollContents2 {},
                Children::spawn(SpawnWith(children_spawner))
            )],
        )
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
pub struct ScrollContainer;

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
    ..default()
}, BackgroundColor(Color::Srgba(GRAY_200)))]
#[component(on_insert=Self::on_insert)]
pub struct ScrollArea;

impl ScrollArea {
    pub fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let mut node = world.get_mut::<Node>(context.entity).unwrap();
        node.width = Val::Percent(100.0);
        node.height = Val::Percent(100.0);
        node.overflow = Overflow::scroll();
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
    scroll_areas: Query<(Entity, &ScrollPosition), (With<ScrollArea>, Changed<ScrollPosition>)>,
    // mut scrolled_node_query: Query<&mut ScrollPosition>,
) {
    for (e, sp) in scroll_areas {
        println!("scroll area {e}");
        // let sp = scrolled_node_query.get_mut(e).unwrap();
        println!("{sp:?}");
    }
}
