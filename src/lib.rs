#![expect(
    clippy::type_complexity,
    reason = "Query types can be complex without being difficult to understand"
)]
pub mod card;
pub mod color_palette;
pub mod color_palette_combo;
pub mod color_push_button;
pub mod combo;
pub mod float;
pub mod label;
pub mod list;
pub mod numeric_stepper;
pub mod push_button;
pub mod scroll_area;
pub mod select;
pub mod stack;
pub mod tailwind;
pub mod theme;

use bevy::prelude::*;

use crate::card::CardPlugin;
use crate::color_palette::ColorPalettePlugin;
use crate::color_palette_combo::ColorPaletteComboPlugin;
use crate::color_push_button::ColorPushButtonPlugin;
use crate::combo::ComboPlugin;
use crate::float::FloatPlugin;
use crate::label::LabelPlugin;
use crate::list::ListPlugin;
use crate::numeric_stepper::NumericStepperPlugin;
use crate::push_button::PushButtonPlugin;
use crate::scroll_area::ScrollAreaPlugin;
use crate::select::ItemsPlugin;
use crate::stack::StackPlugin;
use crate::theme::Theme;

#[derive(Debug, Copy, Clone)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>()
            .add_plugins(ListPlugin)
            .add_plugins(PushButtonPlugin)
            .add_plugins(FloatPlugin)
            .add_plugins(ComboPlugin)
            .add_plugins(ScrollAreaPlugin)
            .add_plugins(StackPlugin)
            .add_plugins(ColorPalettePlugin)
            .add_plugins(ColorPaletteComboPlugin)
            .add_plugins(ColorPushButtonPlugin)
            .add_plugins(ItemsPlugin)
            .add_plugins(LabelPlugin)
            .add_plugins(CardPlugin)
            .add_plugins(NumericStepperPlugin)
            .add_systems(PostStartup, setup);
    }
}

fn setup(mut commands: Commands, root_node: Query<Entity, (With<Node>, Without<ChildOf>)>) {
    let Ok(root) = root_node.single() else {
        panic!("A root node is required");
    };

    commands.entity(root).observe_and_transform_mouse_events();
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
