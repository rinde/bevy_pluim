#![expect(
    clippy::type_complexity,
    reason = "Query types can be complex without being difficult to understand"
)]
pub mod combo;
pub mod float;
pub mod list;
pub mod push_button;
pub mod scroll_area;
pub mod select;
pub mod tailwind;
pub mod theme;

use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::PostStartup;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::event::Event;
use bevy::ecs::hierarchy::ChildOf;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::Commands;
use bevy::ecs::system::EntityCommands;
use bevy::ecs::system::Query;
use bevy::ecs::world::DeferredWorld;
use bevy::ecs::world::EntityWorldMut;
use bevy::picking::events::Out;
use bevy::picking::events::Over;
use bevy::picking::events::Pointer;
use bevy::picking::events::Pressed;
use bevy::picking::events::Released;
use bevy::reflect::Reflect;
use bevy::ui::Node;

use crate::combo::ComboPlugin;
use crate::float::FloatPlugin;
use crate::list::ListPlugin;
use crate::push_button::PushButtonPlugin;
use crate::scroll_area::ScrollAreaPlugin;
use crate::theme::Theme;

#[derive(Debug, Copy, Clone)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Widgets>()
            .init_resource::<Theme>()
            .add_plugins(ListPlugin)
            .add_plugins(PushButtonPlugin)
            .add_plugins(FloatPlugin)
            .add_plugins(ComboPlugin)
            .add_plugins(ScrollAreaPlugin)
            .add_systems(PostStartup, setup);
    }
}

fn setup(mut commands: Commands, root_node: Query<Entity, (With<Node>, Without<ChildOf>)>) {
    let Ok(root) = root_node.single() else {
        panic!("A root node is required");
    };

    commands.entity(root).observe_and_transform_mouse_events();
}

#[derive(Resource, Default, Copy, Clone, Debug)]
pub struct Widgets {
    id: u32,
}

impl Widgets {
    pub const fn next_id(&mut self) -> WidgetId {
        let id = WidgetId(self.id);
        self.id += 1;
        id
    }
}

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct WidgetId(u32);

/// Checks for the existence of a component on an entity or a another entity
/// with the same `WidgetId`
#[must_use]
pub fn get_anywhere<'a, T: Component>(
    world: &'a DeferredWorld,
    context: &HookContext,
    id: WidgetId,
) -> Option<(Entity, &'a T)> {
    let same = world
        .entity(context.entity)
        .get::<T>()
        .map(|t| (context.entity, t));
    if same.is_some() {
        return same;
    }
    let state = world.try_query_filtered::<(Entity, &WidgetId, &T), With<T>>();
    state
        .and_then(|mut s| {
            s.iter(world)
                .find(|(_, component_id, _)| (**component_id == id))
        })
        .map(|(entity, _, comp)| (entity, comp))
}

pub fn get_or_insert_id(world: &mut DeferredWorld, context: &HookContext) -> WidgetId {
    world
        .entity(context.entity)
        .get::<WidgetId>()
        .copied()
        .unwrap_or_else(|| {
            let mut widgets = world
                .get_resource_mut::<Widgets>()
                .expect("Missing Widgets resource, make sure the WidgetsPlugin is added");
            let id = widgets.next_id();
            world.commands().entity(context.entity).insert(id);
            id
        })
}

#[derive(Event, Copy, Clone, Debug)]
enum MouseEvent {
    Over,
    Pressed,
    Released,
    Out,
}

trait ToMouseEvent {
    fn to_mouse_event(&self) -> MouseEvent;
}

impl ToMouseEvent for Over {
    fn to_mouse_event(&self) -> MouseEvent {
        MouseEvent::Over
    }
}

impl ToMouseEvent for Pressed {
    fn to_mouse_event(&self) -> MouseEvent {
        MouseEvent::Pressed
    }
}

impl ToMouseEvent for Released {
    fn to_mouse_event(&self) -> MouseEvent {
        MouseEvent::Released
    }
}

impl ToMouseEvent for Out {
    fn to_mouse_event(&self) -> MouseEvent {
        MouseEvent::Out
    }
}

fn observer_transformer<E>(trigger: Trigger<Pointer<E>>, mut commands: Commands)
where
    E: std::fmt::Debug + Clone + Reflect + ToMouseEvent,
{
    let e: MouseEvent = trigger.event().event.to_mouse_event();
    commands.trigger_targets(e, trigger.target);
}

trait EntityCommandsExt {
    fn observe_and_transform_mouse_events(&mut self) -> &mut Self;
}

impl EntityCommandsExt for EntityCommands<'_> {
    fn observe_and_transform_mouse_events(&mut self) -> &mut Self {
        self.observe(observer_transformer::<Over>)
            .observe(observer_transformer::<Pressed>)
            .observe(observer_transformer::<Released>)
            .observe(observer_transformer::<Out>);
        self
    }
}

impl EntityCommandsExt for EntityWorldMut<'_> {
    fn observe_and_transform_mouse_events(&mut self) -> &mut Self {
        self.observe(observer_transformer::<Over>)
            .observe(observer_transformer::<Pressed>)
            .observe(observer_transformer::<Released>)
            .observe(observer_transformer::<Out>);
        self
    }
}
