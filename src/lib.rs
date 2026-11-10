//! A reusable Bevy UI widget library built on Bevy's native `Node` UI: buttons, lists,
//! combos, scroll areas, tabs, breadcrumbs, colour palettes, a numeric stepper and a
//! slider, all themed from one [`Theme`] resource.
//!
//! Add [`WidgetsPlugin`] to an app and spawn exactly one root [`Node`] for the app's UI.
//!
//! The crate takes Bevy types only: anything that knows what a picked item
//! *means* belongs in the application that uses the widgets.

pub mod badge;
pub mod breadcrumbs;
pub mod card;
pub mod color_palette;
pub mod color_palette_combo;
pub mod color_push_button;
pub mod combo;
pub mod divider;
pub mod filter_chips;
pub mod filter_combo;
pub mod float;
pub(crate) mod focus;
pub mod gallery;
pub mod icon;
pub mod icon_button;
pub(crate) mod indicator;
pub mod kbd;
pub mod label;
pub mod light_dark_toggle;
pub mod link;
pub mod list;
pub mod numeric_stepper;
pub mod panel;
pub mod pill;
pub(crate) mod popup;
pub mod push_button;
pub mod scroll_area;
pub mod segmented_control;
pub mod select;
pub mod setting_row;
pub mod slider;
pub mod stack;
pub mod tab_bar;
pub mod tailwind;
pub mod text_field;
pub mod theme;
pub mod theme_editor;
pub mod toast;
pub mod toolbar;
pub mod tooltip;
pub mod virtual_list;

use bevy::ecs::event::PropagateEntityTrigger;
use bevy::input_focus::tab_navigation::{TabGroup, TabNavigationPlugin};
use bevy::input_focus::{InputDispatchPlugin, InputFocusPlugin};
use bevy::picking::cursor::CursorIconPlugin;
use bevy::picking::events::{PointerEvent, PointerTraversal};
use bevy::prelude::*;
use bevy::ui_widgets::{
    ButtonPlugin, ListBoxPlugin, ScrollbarPlugin, SliderPlugin as HeadlessSliderPlugin, TabPlugin,
};

use crate::badge::BadgePlugin;
use crate::breadcrumbs::BreadCrumbsPlugin;
use crate::card::CardPlugin;
use crate::color_palette::ColorPalettePlugin;
use crate::color_palette_combo::ColorPaletteComboPlugin;
use crate::color_push_button::ColorPushButtonPlugin;
use crate::combo::ComboPlugin;
use crate::divider::DividerPlugin;
use crate::filter_chips::FilterChipsPlugin;
use crate::filter_combo::FilterComboPlugin;
use crate::float::FloatPlugin;
use crate::focus::FocusPlugin;
use crate::gallery::GalleryPlugin;
use crate::icon::IconPlugin;
use crate::icon_button::IconButtonPlugin;
use crate::kbd::KbdPlugin;
use crate::label::LabelPlugin;
use crate::light_dark_toggle::LightDarkTogglePlugin;
use crate::link::LinkPlugin;
use crate::list::ListPlugin;
use crate::numeric_stepper::NumericStepperPlugin;
use crate::panel::PanelPlugin;
use crate::pill::PillPlugin;
use crate::push_button::PushButtonPlugin;
use crate::scroll_area::ScrollAreaPlugin;
use crate::segmented_control::SegmentedControlPlugin;
use crate::select::ItemsPlugin;
use crate::setting_row::SettingRowPlugin;
use crate::slider::SliderPlugin;
use crate::stack::StackPlugin;
use crate::tab_bar::TabBarPlugin;
use crate::text_field::TextFieldPlugin;
use crate::theme::{Theme, ThemePlugin};
use crate::theme_editor::ThemeEditorPlugin;
use crate::toast::ToastPlugin;
use crate::toolbar::ToolbarPlugin;
use crate::tooltip::TooltipPlugin;
use crate::virtual_list::VirtualListPlugin;

/// Adds every widget of this crate, and the [`Theme`] resource, to an app.
///
/// The `ButtonPlugin`, `ListBoxPlugin`, `ScrollbarPlugin`, `SliderPlugin` and `TabPlugin` of
/// `bevy_ui_widgets` are added in [`Plugin::finish`] when the app does not have them by
/// then, so it does not matter whether `UiWidgetsPlugins` (or `DefaultPlugins`) is added
/// before or after this plugin. An app that is driven by hand with `App::update` instead
/// of `App::run` must call `App::finish` first, or its buttons, lists, scroll bars,
/// sliders and tab bars get no observers.
#[derive(Debug, Copy, Clone)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>()
            .add_plugins(FocusPlugin)
            .add_plugins(ListPlugin)
            .add_plugins(VirtualListPlugin)
            .add_plugins(PushButtonPlugin)
            .add_plugins(FloatPlugin)
            .add_plugins(ComboPlugin)
            .add_plugins(ThemeEditorPlugin)
            .add_plugins(FilterComboPlugin)
            .add_plugins(ScrollAreaPlugin)
            .add_plugins(SegmentedControlPlugin)
            .add_plugins(LightDarkTogglePlugin)
            .add_plugins(SettingRowPlugin)
            .add_plugins(TabBarPlugin)
            .add_plugins(StackPlugin)
            .add_plugins(ColorPalettePlugin)
            .add_plugins(ColorPaletteComboPlugin)
            .add_plugins(ColorPushButtonPlugin)
            .add_plugins(ItemsPlugin)
            .add_plugins(LabelPlugin)
            .add_plugins(CardPlugin)
            .add_plugins(DividerPlugin)
            .add_plugins(IconPlugin)
            .add_plugins(IconButtonPlugin)
            .add_plugins(KbdPlugin)
            .add_plugins(PillPlugin)
            .add_plugins(BadgePlugin)
            .add_plugins(FilterChipsPlugin)
            .add_plugins(ToolbarPlugin)
            .add_plugins(TooltipPlugin)
            .add_plugins(ToastPlugin)
            .add_plugins(GalleryPlugin)
            .add_plugins(NumericStepperPlugin)
            .add_plugins(SliderPlugin)
            .add_plugins(PanelPlugin)
            .add_plugins(TextFieldPlugin)
            .add_plugins(ThemePlugin)
            .add_plugins(BreadCrumbsPlugin)
            .add_plugins(LinkPlugin)
            .add_systems(PostStartup, setup)
            .add_observer(add_ui_root)
            .add_observer(drop_ui_root_when_parented)
            .add_observer(observer_transformer::<PointerOver>)
            .add_observer(observer_transformer::<PointerPress>)
            .add_observer(observer_transformer::<PointerRelease>)
            .add_observer(observer_transformer::<PointerClick>)
            .add_observer(observer_transformer::<PointerDrag>)
            .add_observer(observer_transformer::<PointerOut>);
    }

    /// `DefaultPlugins` and the tester's `HeadlessPlugins` bring Bevy's
    /// `UiWidgetsPlugins` along; a bare `App` does not. Adding the same plugin
    /// twice panics, so the missing ones are added here, once every plugin of the
    /// app has been built and it is known which ones the app has.
    fn finish(&self, app: &mut App) {
        if !app.is_plugin_added::<ButtonPlugin>() {
            app.add_plugins(ButtonPlugin);
        }
        if !app.is_plugin_added::<ListBoxPlugin>() {
            app.add_plugins(ListBoxPlugin);
        }
        if !app.is_plugin_added::<ScrollbarPlugin>() {
            app.add_plugins(ScrollbarPlugin);
        }
        if !app.is_plugin_added::<HeadlessSliderPlugin>() {
            app.add_plugins(HeadlessSliderPlugin);
        }
        if !app.is_plugin_added::<TabPlugin>() {
            app.add_plugins(TabPlugin);
        }
        // The same for keyboard focus: the tester's `HeadlessPlugins` has the first two but
        // not tab navigation.
        if !app.is_plugin_added::<InputFocusPlugin>() {
            app.add_plugins(InputFocusPlugin);
        }
        if !app.is_plugin_added::<InputDispatchPlugin>() {
            app.add_plugins(InputDispatchPlugin);
        }
        if !app.is_plugin_added::<TabNavigationPlugin>() {
            app.add_plugins(TabNavigationPlugin);
        }
        // Widgets name the cursor they want with `EntityCursor`; this plugin copies it to the
        // window.
        if !app.is_plugin_added::<CursorIconPlugin>() {
            app.add_plugins(CursorIconPlugin);
        }
    }
}

fn add_ui_root(
    t: On<Add<Node>>,
    app_root: Query<(), (Without<ChildOf>, Without<UiTargetCamera>)>,
    mut commands: Commands,
) {
    let entity = t.event().event_target();
    if app_root.contains(entity) {
        commands.entity(entity).insert(UiRoot);
    }
}

fn drop_ui_root_when_parented(t: On<Insert<ChildOf>>, mut commands: Commands) {
    commands.entity(t.event().event_target()).remove::<UiRoot>();
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
/// the same bundle. A node that gets its parent after its [`Node`] (a scene
/// writes `ChildOf` last) loses the marker again when the [`ChildOf`] is
/// inserted. It is the one non-modal [`TabGroup`],
/// so Tab moves through the widgets below it. If the app does not end up with exactly one such root at
/// startup, an error is logged.
#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("UiRoot"), TabGroup::new(0))]
pub struct UiRoot;

/// What the pointer did to a widget.
#[derive(Copy, Clone, Debug)]
enum MouseKind {
    Over,
    Pressed,
    Released,
    /// The pointer pressed and released on the widget without moving off it.
    Click(PointerButton),
    /// The pointer moved while a button was held down on the widget.
    Drag,
    Out,
}

/// A pointer interaction with `entity`, re-triggered from Bevy's pointer events
/// ([`PointerEvent`]s such as `PointerPress`).
#[derive(EntityEvent, Copy, Clone, Debug)]
struct MouseEvent {
    entity: Entity,
    kind: MouseKind,
    /// For [`MouseKind::Drag`], how far the pointer has moved since the press, in logical
    /// pixels; zero for every other kind.
    distance: Vec2,
    /// The button of a press, release, click or drag; `None` for a pointer moving over or out.
    /// A touch counts as [`PointerButton::Primary`].
    button: Option<PointerButton>,
}

trait ToMouseEvent {
    fn to_mouse_kind(&self) -> MouseKind;

    /// How far the pointer has been dragged since the press; zero unless this is a drag.
    fn distance(&self) -> Vec2 {
        Vec2::ZERO
    }

    /// The button that caused the event, if it has one.
    fn button(&self) -> Option<PointerButton> {
        None
    }
}

impl ToMouseEvent for PointerOver {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Over
    }
}

impl ToMouseEvent for PointerPress {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Pressed
    }

    fn button(&self) -> Option<PointerButton> {
        Some(self.button)
    }
}

impl ToMouseEvent for PointerRelease {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Released
    }

    fn button(&self) -> Option<PointerButton> {
        Some(self.button)
    }
}

impl ToMouseEvent for PointerClick {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Click(self.button)
    }

    fn button(&self) -> Option<PointerButton> {
        Some(self.button)
    }
}

impl ToMouseEvent for PointerDrag {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Drag
    }

    fn button(&self) -> Option<PointerButton> {
        Some(self.button)
    }

    fn distance(&self) -> Vec2 {
        self.distance
    }
}

impl ToMouseEvent for PointerOut {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Out
    }
}

/// Re-triggers a pointer event `E` as a [`MouseEvent`]. The `Trigger` bound says that
/// `E` bubbles through [`PointerTraversal`], as every pointer event does, which is what
/// gives `On<E>` its `original_event_target`.
fn observer_transformer<E>(trigger: On<E>, mut commands: Commands)
where
    E: PointerEvent
        + ToMouseEvent
        + for<'t> EntityEvent<Trigger<'t> = PropagateEntityTrigger<true, E, PointerTraversal>>,
{
    // The event bubbles up through the hierarchy, rewriting its own target at
    // every hop; the widget that was actually hit is the original target, and
    // only the first hop re-triggers, so one event is one `MouseEvent`.
    //
    // This is a global observer rather than one on `UiRoot` because Bevy's
    // `Button` stops `PointerPress`, `PointerRelease`, `PointerClick`,
    // `PointerDragEnd` and `PointerCancel` from bubbling, which would keep
    // them from ever reaching the root.
    let original = trigger.original_event_target();
    if trigger.event_target() == original {
        commands.trigger(MouseEvent {
            entity: original,
            kind: trigger.event().to_mouse_kind(),
            distance: trigger.event().distance(),
            button: trigger.event().button(),
        });
    }
}
