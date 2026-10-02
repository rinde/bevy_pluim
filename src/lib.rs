//! A reusable Bevy UI widget library built on Bevy's native `Node` UI: buttons, lists,
//! combos, scroll areas, tabs, breadcrumbs, colour palettes and a numeric stepper,
//! all themed from one [`Theme`] resource.
//!
//! Add [`WidgetsPlugin`] to an app and spawn exactly one root [`Node`].
//!
//! The crate takes Bevy types only: anything that knows what a picked item
//! *means* belongs in the application that uses the widgets.

pub mod breadcrumbs;
pub mod card;
pub mod color_palette;
pub mod color_palette_combo;
pub mod color_push_button;
pub mod combo;
pub mod float;
pub mod label;
pub mod link;
pub mod list;
pub mod numeric_stepper;
pub mod push_button;
pub mod scroll_area;
pub mod select;
pub mod stack;
pub mod tailwind;
pub mod theme;

use bevy::ecs::entity_disabling::Disabled;
use bevy::prelude::*;

use crate::breadcrumbs::BreadCrumbsPlugin;
use crate::card::CardPlugin;
use crate::color_palette::ColorPalettePlugin;
use crate::color_palette_combo::ColorPaletteComboPlugin;
use crate::color_push_button::ColorPushButtonPlugin;
use crate::combo::ComboPlugin;
use crate::float::FloatPlugin;
use crate::label::LabelPlugin;
use crate::link::LinkPlugin;
use crate::list::ListPlugin;
use crate::numeric_stepper::NumericStepperPlugin;
use crate::push_button::PushButtonPlugin;
use crate::scroll_area::ScrollAreaPlugin;
use crate::select::ItemsPlugin;
use crate::stack::StackPlugin;
use crate::theme::{Theme, ThemePlugin};

/// Adds every widget of this crate, and the [`Theme`] resource, to an app.
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
            .add_plugins(ThemePlugin)
            .add_plugins(BreadCrumbsPlugin)
            .add_plugins(LinkPlugin)
            .add_systems(PostStartup, setup)
            .add_observer(add_ui_root);
    }
}

fn add_ui_root(
    t: On<Add, Node>,
    has_childof: Query<(Has<ChildOf>, Has<Disabled>)>,
    mut commands: Commands,
) {
    let Ok((has_parent, _)) = has_childof.get(t.event().event_target()) else {
        return;
    };
    if !has_parent {
        commands
            .entity(t.event().event_target())
            .insert(UiRoot)
            .observe_and_transform_mouse_events();
    }
}

fn setup(root_node: Query<Entity, (With<Node>, Without<ChildOf>)>) {
    assert!(
        root_node.iter().count() == 1,
        "Exactly one root node is required"
    );
}

/// Marks the one node without a parent that all other UI hangs off.
///
/// Inserted automatically on any [`Node`] that has no parent.
#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("UiRoot"))]
pub struct UiRoot;

/// What the pointer did to a widget.
#[derive(Copy, Clone, Debug)]
enum MouseKind {
    Over,
    Pressed,
    Released,
    Out,
}

/// A pointer interaction with `entity`, re-triggered from Bevy's `Pointer<_>` events.
#[derive(EntityEvent, Copy, Clone, Debug)]
struct MouseEvent {
    entity: Entity,
    kind: MouseKind,
}

trait ToMouseEvent {
    fn to_mouse_kind(&self) -> MouseKind;
}

impl ToMouseEvent for Over {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Over
    }
}

impl ToMouseEvent for Press {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Pressed
    }
}

impl ToMouseEvent for Release {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Released
    }
}

impl ToMouseEvent for Out {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Out
    }
}

fn observer_transformer<E>(trigger: On<Pointer<E>>, mut commands: Commands)
where
    E: std::fmt::Debug + Clone + Reflect + ToMouseEvent,
{
    // The event bubbles up to the root, rewriting its own target at every hop;
    // the widget that was actually hit is the original target.
    commands.trigger(MouseEvent {
        entity: trigger.original_event_target(),
        kind: trigger.event().event.to_mouse_kind(),
    });
}

trait EntityCommandsExt {
    fn observe_and_transform_mouse_events(&mut self) -> &mut Self;
}

impl EntityCommandsExt for EntityCommands<'_> {
    fn observe_and_transform_mouse_events(&mut self) -> &mut Self {
        self.observe(observer_transformer::<Over>)
            .observe(observer_transformer::<Press>)
            .observe(observer_transformer::<Release>)
            .observe(observer_transformer::<Out>);
        self
    }
}

impl EntityCommandsExt for EntityWorldMut<'_> {
    fn observe_and_transform_mouse_events(&mut self) -> &mut Self {
        self.observe(observer_transformer::<Over>)
            .observe(observer_transformer::<Press>)
            .observe(observer_transformer::<Release>)
            .observe(observer_transformer::<Out>);
        self
    }
}
