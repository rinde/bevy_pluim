//! Playground crate demonstrating Bevy UI components with TailwindCSS-inspired
//! styling.

pub mod colors;
pub mod combo2;
pub mod combobox;
pub mod selectable_items;
pub mod theme;

use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::resource::Resource;
use bevy::ecs::world::FromWorld;
use bevy::ecs::world::World;
pub use combobox::*;

use crate::selectable_items::SelectableItemsPlugin;
use crate::theme::Theme;
use crate::theme::on_theme_change;

#[derive(Debug, Copy, Clone)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Widgets>()
            .init_resource::<Theme>()
            .add_plugins(SelectableItemsPlugin)
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

#[derive(Component, Copy, Clone, Debug, PartialEq, Eq)]
pub struct WidgetId(u32);

impl FromWorld for WidgetId {
    fn from_world(world: &mut World) -> Self {
        let mut widgets = world
            .get_resource_mut::<Widgets>()
            .expect("No Widgets resource found");
        widgets.next_id()
    }
}
