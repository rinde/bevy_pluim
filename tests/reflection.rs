//! The widget components and the `Theme` resource are in the type registry of an app with
//! `WidgetsPlugin`, so that tools which look types up by name (such as scenario files) can find
//! them. A refactor that drops a derive fails here rather than in a scenario.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "tests may abort loudly"
)]

use core::any::TypeId;

use bevy::MinimalPlugins;
use bevy::ecs::reflect::{AppTypeRegistry, ReflectComponent, ReflectResource};
use bevy::prelude::*;
use bevy::reflect::TypeRegistry;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::badge::{Badge, BadgeRef, BadgeTone};
use bevy_pluim::breadcrumbs::{BreadCrumb, BreadCrumbs};
use bevy_pluim::card::Card;
use bevy_pluim::color_palette::ColorPalette;
use bevy_pluim::color_palette_combo::{
    self, ColorPaletteCombo, ColorPaletteComboButton, ColorPaletteComboFloat, ColorPaletteComboRefs,
};
use bevy_pluim::color_push_button::{ColorPushButton, ColorSquare};
use bevy_pluim::combo::{Combo, ComboButton, ComboList, ComboPlaceholder, ComboRefs, ComboState};
use bevy_pluim::divider::Divider;
use bevy_pluim::filter_chips::{ChipCount, ChipLabel, ChipRefs, ChipView, FilterChips};
use bevy_pluim::filter_combo::{FilterCombo, FilterComboRefs};
use bevy_pluim::float::Float;
use bevy_pluim::gallery::{Gallery, GalleryCardRefs, GalleryCardView};
use bevy_pluim::icon::Icon;
use bevy_pluim::icon_button::{IconButton, ToggleState};
use bevy_pluim::kbd::Kbd;
use bevy_pluim::label::Label;
use bevy_pluim::light_dark_toggle::{
    LightDarkOption, LightDarkToggle, SystemTheme, ThemePreference,
};
use bevy_pluim::link::Link;
use bevy_pluim::list::{CornerStyle, ItemNote, ItemView, List, ListLayout};
use bevy_pluim::numeric_stepper::{NumericStepper, UIntValue, Value, ValueRef};
use bevy_pluim::panel::Panel;
use bevy_pluim::pill::{Pill, PillTone};
use bevy_pluim::push_button::{ButtonSize, IconPlacement, PushButton};
use bevy_pluim::scroll_area::{ClearOfCorners, ScrollArea, ScrollBar, ScrollContents};
use bevy_pluim::segmented_control::{
    ItemIcon, Segment, SegmentIcon, SegmentLabels, SegmentedControl,
};
use bevy_pluim::setting_row::{SettingRow, SettingRows};
use bevy_pluim::slider::{Slider, SliderRefs};
use bevy_pluim::stack::{Stack, StackItem};
use bevy_pluim::tab_bar::{TabBar, TabBarTab};
use bevy_pluim::text_field::{TextField, TextFieldRefs};
use bevy_pluim::theme::{Theme, ThemeColors};
use bevy_pluim::theme_editor::{
    ThemeColorSlot, ThemeColorSwatch, ThemeEditor, ThemeEditorRefs, ThemeParameterEditor,
};
use bevy_pluim::toast::{Toast, ToastPlacement, ToastStack};
use bevy_pluim::toolbar::{Toolbar, ToolbarButton, ToolbarButtonRefs, ToolbarLabels};
use bevy_pluim::tooltip::{Tooltip, TooltipPopup};
use bevy_pluim::virtual_list::{VirtualList, VirtualListRow};

fn app_with_widgets() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(WidgetsPlugin);
    app
}

/// Runs `check` on the type registry of an app with `WidgetsPlugin`.
fn with_registry(check: impl FnOnce(&TypeRegistry)) {
    let app = app_with_widgets();
    let registry = app
        .world()
        .get_resource::<AppTypeRegistry>()
        .expect("an app has a type registry");
    check(&registry.read());
}

fn assert_component<T: Component>(registry: &TypeRegistry) {
    let name = core::any::type_name::<T>();
    let registration = registry.get(TypeId::of::<T>());
    assert!(registration.is_some(), "{name} is not in the type registry");
    let registration = registration.unwrap();
    assert!(
        registration.data::<ReflectComponent>().is_some(),
        "{name} is registered but not as a component"
    );
}

fn assert_registered<T: 'static>(registry: &TypeRegistry) {
    let name = core::any::type_name::<T>();
    assert!(
        registry.get(TypeId::of::<T>()).is_some(),
        "{name} is not in the type registry"
    );
}

#[test]
fn the_widget_components_are_in_the_type_registry() {
    with_registry(|registry| {
        assert_component::<PushButton>(registry);
        assert_component::<ButtonSize>(registry);
        assert_component::<IconPlacement>(registry);
        assert_component::<Combo>(registry);
        assert_component::<ComboButton>(registry);
        assert_component::<ComboList>(registry);
        assert_component::<List>(registry);
        assert_component::<ListLayout>(registry);
        assert_component::<CornerStyle>(registry);
        assert_component::<ItemView>(registry);
        assert_component::<ItemNote>(registry);
        assert_component::<Link>(registry);
        assert_component::<BreadCrumbs>(registry);
        assert_component::<BreadCrumb>(registry);
        assert_component::<ColorPalette>(registry);
        assert_component::<ColorPaletteCombo>(registry);
        assert_component::<FilterCombo>(registry);
        assert_component::<ThemeEditor>(registry);
        assert_component::<ThemeColorSlot>(registry);
        assert_component::<ThemeColorSwatch>(registry);
        assert_component::<ThemeParameterEditor>(registry);
        assert_component::<ColorPushButton>(registry);
        assert_component::<ColorSquare>(registry);
        assert_component::<NumericStepper>(registry);
        assert_component::<Slider>(registry);
        assert_component::<SliderRefs>(registry);
        assert_component::<ValueRef>(registry);
        assert_component::<ScrollArea>(registry);
        assert_component::<ClearOfCorners>(registry);
        assert_component::<ScrollContents>(registry);
        assert_component::<ScrollBar>(registry);
        assert_component::<Stack>(registry);
        assert_component::<StackItem>(registry);
        assert_component::<Card>(registry);
        assert_component::<Divider>(registry);
        assert_component::<Icon>(registry);
        assert_component::<SegmentedControl>(registry);
        assert_component::<LightDarkToggle>(registry);
        assert_component::<LightDarkOption>(registry);
        assert_component::<SettingRow>(registry);
        assert_component::<SettingRows>(registry);
        assert_component::<Segment>(registry);
        assert_component::<SegmentIcon>(registry);
        assert_component::<SegmentLabels>(registry);
        assert_component::<ItemIcon>(registry);
        assert_component::<TabBar>(registry);
        assert_component::<TabBarTab>(registry);
        assert_component::<IconButton>(registry);
        assert_component::<Panel>(registry);
        assert_component::<ToggleState>(registry);
        assert_component::<Kbd>(registry);
        assert_component::<Pill>(registry);
        assert_component::<Badge>(registry);
        assert_component::<BadgeRef>(registry);
        assert_registered::<BadgeTone>(registry);
        assert_registered::<PillTone>(registry);
        assert_component::<FilterChips>(registry);
        assert_component::<ChipView>(registry);
        assert_component::<ChipRefs>(registry);
        assert_component::<ChipLabel>(registry);
        assert_component::<ChipCount>(registry);
        assert_component::<Gallery>(registry);
        assert_component::<GalleryCardView>(registry);
        assert_component::<GalleryCardRefs>(registry);
        assert_component::<Toolbar>(registry);
        assert_component::<ToolbarButton>(registry);
        assert_component::<ToolbarButtonRefs>(registry);
        assert_component::<ToolbarLabels>(registry);
        assert_component::<VirtualList>(registry);
        assert_component::<VirtualListRow>(registry);
        assert_component::<Tooltip>(registry);
        assert_component::<TooltipPopup>(registry);
        assert_component::<Toast>(registry);
        assert_component::<ToastStack>(registry);
        assert_component::<TextField>(registry);
        assert_component::<TextFieldRefs>(registry);
        assert_component::<Label>(registry);
        assert_component::<Float>(registry);
    });
}

#[test]
fn the_widget_states_are_in_the_type_registry() {
    with_registry(|registry| {
        assert_component::<ComboState>(registry);
        assert_component::<ComboPlaceholder>(registry);
        assert_component::<ComboRefs>(registry);
        assert_component::<FilterComboRefs>(registry);
        assert_component::<ThemeEditorRefs>(registry);
        assert_component::<ColorPaletteComboRefs>(registry);
        assert_component::<ColorPaletteComboButton>(registry);
        assert_component::<ColorPaletteComboFloat>(registry);
        assert_component::<color_palette_combo::ColorPaletteComboState>(registry);
        assert_component::<Value>(registry);
        assert_registered::<UIntValue<u8>>(registry);
        assert_registered::<UIntValue<u16>>(registry);
        assert_registered::<UIntValue<u32>>(registry);
    });
}

#[test]
fn the_theme_is_in_the_type_registry_as_a_resource() {
    with_registry(|registry| {
        let theme = registry
            .get(TypeId::of::<Theme>())
            .expect("Theme is in the type registry");
        assert!(theme.data::<ReflectResource>().is_some());
        assert!(registry.contains(TypeId::of::<ThemeColors>()));
    });
}

#[test]
fn the_toast_placement_is_in_the_type_registry_as_a_resource() {
    with_registry(|registry| {
        let registration = registry
            .get(TypeId::of::<ToastPlacement>())
            .expect("ToastPlacement is in the type registry");
        assert!(registration.data::<ReflectResource>().is_some());
    });
}

#[test]
fn the_theme_preference_and_the_system_theme_are_in_the_type_registry_as_resources() {
    with_registry(|registry| {
        for id in [TypeId::of::<ThemePreference>(), TypeId::of::<SystemTheme>()] {
            let registration = registry.get(id).expect("in the type registry");
            assert!(registration.data::<ReflectResource>().is_some());
        }
    });
}

#[test]
fn a_widget_is_found_by_its_short_type_path() {
    with_registry(|registry| {
        let found = registry
            .get_with_short_type_path("PushButton")
            .expect("PushButton resolves by short type path");
        assert_eq!(found.type_id(), TypeId::of::<PushButton>());
    });
}
