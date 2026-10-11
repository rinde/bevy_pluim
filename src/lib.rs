//! A reusable Bevy UI widget library built on Bevy's native `Node` UI: buttons, lists,
//! combos, scroll areas, tabs, breadcrumbs, colour palettes and a numeric stepper,
//! all themed from one [`Theme`] resource.
//!
//! Add [`WidgetsPlugin`] to an app and spawn exactly one root [`Node`] for the app's UI.
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

use bevy::prelude::*;
use bevy::ui_widgets::{ButtonPlugin, ListBoxPlugin};

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
        // `DefaultPlugins` and the tester's `HeadlessPlugins` bring Bevy's
        // `UiWidgetsPlugins` along; a bare `App` does not. Adding the observers
        // twice would panic, so add them only when they are missing.
        if !app.is_plugin_added::<ButtonPlugin>() {
            app.add_plugins(ButtonPlugin);
        }
        if !app.is_plugin_added::<ListBoxPlugin>() {
            app.add_plugins(ListBoxPlugin);
        }
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
            .add_observer(add_ui_root)
            .add_observer(observer_transformer::<Over>)
            .add_observer(observer_transformer::<Press>)
            .add_observer(observer_transformer::<Release>)
            .add_observer(observer_transformer::<Out>);
    }
}

fn add_ui_root(
    t: On<Add, Node>,
    app_root: Query<(), (Without<ChildOf>, Without<UiTargetCamera>)>,
    mut commands: Commands,
) {
    let entity = t.event().event_target();
    if app_root.contains(entity) {
        commands.entity(entity).insert(UiRoot);
    }
}

fn setup(roots: Query<(), With<UiRoot>>) {
    let count = roots.iter().count();
    if count != 1 {
        error!(
            "Exactly one root node is required for the app's UI, found {count}; \
             root nodes aimed at another camera (`UiTargetCamera`) are not counted"
        );
    }
}

/// Marks the one node without a parent that all other UI hangs off.
///
/// Inserted automatically on a [`Node`] that has no parent and no
/// [`UiTargetCamera`], when the node is added. Root nodes aimed at another camera
/// (a second window's UI, an overlay) belong to someone else and are left alone.
/// A [`UiTargetCamera`] inserted after the [`Node`] is not seen, so insert it in
/// the same bundle. If the app does not end up with exactly one such root at
/// startup, an error is logged.
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
    // The event bubbles up through the hierarchy, rewriting its own target at
    // every hop; the widget that was actually hit is the original target, and
    // only the first hop re-triggers, so one event is one `MouseEvent`.
    //
    // This is a global observer rather than one on `UiRoot` because Bevy's
    // `Button` stops `Press` and `Release` from bubbling, which would keep them
    // from ever reaching the root.
    let original = trigger.original_event_target();
    if trigger.event_target() == original {
        commands.trigger(MouseEvent {
            entity: original,
            kind: trigger.event().event.to_mouse_kind(),
        });
    }
}
