#![expect(
    clippy::type_complexity,
    reason = "Query types can be complex without being difficult to understand"
)]
pub mod combo;
pub mod float;
pub mod list;
pub mod push_button;
pub mod select;
pub mod theme;

use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::event::Event;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::Commands;
use bevy::ecs::system::EntityCommands;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::events::Out;
use bevy::picking::events::Over;
use bevy::picking::events::Pointer;
use bevy::picking::events::Pressed;
use bevy::picking::events::Released;
use bevy::reflect::Reflect;

use crate::combo::ComboPlugin;
use crate::float::FloatPlugin;
use crate::list::ListPlugin;
use crate::push_button::PushButtonPlugin;
use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;
use crate::theme::on_theme_change;

#[derive(Debug, Copy, Clone)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Widgets>()
            .init_resource::<Theme>()
            .add_event::<ThemeChangedEvent>()
            .add_plugins(ListPlugin)
            .add_plugins(PushButtonPlugin)
            .add_plugins(FloatPlugin)
            .add_plugins(ComboPlugin)
            .add_systems(Update, on_theme_change);
    }
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
) -> Option<&'a T> {
    let same = world.entity(context.entity).get::<T>();
    if same.is_some() {
        return same;
    }
    let state = world.try_query_filtered::<(&WidgetId, &T), With<T>>();
    state
        .and_then(|mut s| {
            s.iter(world)
                .find(|(component_id, _)| (**component_id == id))
        })
        .map(|(_, comp)| comp)
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
