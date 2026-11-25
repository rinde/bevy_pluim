//! A Bevy UI widget library on native `Node` UI, themed from one [`Theme`] resource.
//!
//! Add [`WidgetsPlugin`] to an app and spawn exactly one root [`Node`] for the app's UI.

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
pub(crate) mod glyph_images;
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
use bevy::picking::pointer::PointerId;
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
use crate::scroll_area::{ScrollAreaPlugin, TouchPans};
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
/// An app driven by hand with `App::update` must call `App::finish` first, or the
/// `bevy_ui_widgets` plugins that [`Plugin::finish`] adds are missing.
#[derive(Debug, Copy, Clone)]
pub struct WidgetsPlugin;

impl Plugin for WidgetsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Theme>()
            .init_resource::<HoverLock>()
            .init_resource::<glyph_images::GlyphImages>()
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
            .add_observer(observer_transformer::<PointerOut>)
            .add_observer(observer_transformer::<PointerCancel>);
    }

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
        if !app.is_plugin_added::<InputFocusPlugin>() {
            app.add_plugins(InputFocusPlugin);
        }
        if !app.is_plugin_added::<InputDispatchPlugin>() {
            app.add_plugins(InputDispatchPlugin);
        }
        if !app.is_plugin_added::<TabNavigationPlugin>() {
            app.add_plugins(TabNavigationPlugin);
        }
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
/// Added to a parentless [`Node`] without a [`UiTargetCamera`]; insert the camera in the same
/// bundle. Removed when the node gets a [`ChildOf`]. An error is logged at startup unless
/// exactly one root exists.
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
    /// The system took the pointer away, as when a touch is interrupted.
    Cancel,
}

impl MouseKind {
    const fn pressed(self) -> bool {
        matches!(self, Self::Pressed)
    }
}

/// A pointer interaction with `entity`, re-triggered from Bevy's [`PointerEvent`]s.
#[derive(EntityEvent, Copy, Clone, Debug)]
struct MouseEvent {
    entity: Entity,
    kind: MouseKind,
    /// For [`MouseKind::Drag`], the distance moved since the press in logical pixels.
    distance: Vec2,
    /// The button of a press, release, click or drag. A touch counts as [`PointerButton::Primary`].
    button: Option<PointerButton>,
    touch: bool,
    pointer: PointerId,
}

impl MouseEvent {
    /// Whether the hover look shows. The one place that tells touch from mouse: a finger never hovers.
    const fn hovered(&self) -> bool {
        !self.touch && !matches!(self.kind, MouseKind::Out | MouseKind::Cancel)
    }

    const fn pressed(&self) -> bool {
        self.kind.pressed()
    }
}

trait ToMouseEvent {
    fn to_mouse_kind(&self) -> MouseKind;

    fn distance(&self) -> Vec2 {
        Vec2::ZERO
    }

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

impl ToMouseEvent for PointerCancel {
    fn to_mouse_kind(&self) -> MouseKind {
        MouseKind::Cancel
    }
}

/// How far a pointer must move, in logical pixels, before a press is a drag.
pub(crate) const DRAG_THRESHOLD: f32 = 3.0;

/// While set, `MouseKind::Over` is not sent.
#[derive(Resource, Debug, Default)]
struct HoverLock(bool);

/// Re-triggers a pointer event `E` as a [`MouseEvent`].
fn observer_transformer<E>(
    trigger: On<E>,
    hover_lock: Res<HoverLock>,
    pans: Res<TouchPans>,
    mut commands: Commands,
) where
    E: PointerEvent
        + ToMouseEvent
        + for<'t> EntityEvent<Trigger<'t> = PropagateEntityTrigger<true, E, PointerTraversal>>,
{
    // Global, not on `UiRoot`: Bevy's `Button` stops most pointer events from bubbling to the root.
    let original = trigger.original_event_target();
    let kind = trigger.event().to_mouse_kind();
    if hover_lock.0 && matches!(kind, MouseKind::Over) {
        return;
    }
    let pointer = trigger.event().pointer().id;
    if matches!(kind, MouseKind::Click(_)) && pans.panned(pointer) {
        return;
    }
    if trigger.event_target() == original {
        commands.trigger(MouseEvent {
            entity: original,
            kind,
            distance: trigger.event().distance(),
            button: trigger.event().button(),
            touch: matches!(trigger.event().pointer().id, PointerId::Touch(_)),
            pointer: trigger.event().pointer().id,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const fn event(kind: MouseKind, touch: bool) -> MouseEvent {
        MouseEvent {
            entity: Entity::PLACEHOLDER,
            kind,
            distance: Vec2::ZERO,
            button: None,
            touch,
            pointer: PointerId::Mouse,
        }
    }

    #[test]
    fn a_mouse_hovers_until_it_goes_out_or_is_cancelled() {
        assert!(event(MouseKind::Over, false).hovered());
        assert!(event(MouseKind::Released, false).hovered());
        assert!(!event(MouseKind::Out, false).hovered());
        assert!(!event(MouseKind::Cancel, false).hovered());
    }

    #[test]
    fn a_finger_never_hovers_but_presses() {
        for kind in [MouseKind::Over, MouseKind::Pressed, MouseKind::Released] {
            assert!(!event(kind, true).hovered());
        }
        assert!(event(MouseKind::Pressed, true).pressed());
        assert!(!event(MouseKind::Cancel, true).pressed());
        assert!(!event(MouseKind::Released, true).pressed());
    }
}
