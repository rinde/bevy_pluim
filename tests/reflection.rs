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
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::badge::Badge;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::card::Card;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::color_palette_combo::{self, ColorPaletteCombo};
use rin_bevy_ui::color_push_button::{ColorPushButton, ColorSquare};
use rin_bevy_ui::combo::{Combo, ComboButton, ComboList, ComboState};
use rin_bevy_ui::divider::Divider;
use rin_bevy_ui::filter_chips::{ChipView, FilterChips};
use rin_bevy_ui::float::Float;
use rin_bevy_ui::gallery::{Gallery, GalleryCardView};
use rin_bevy_ui::icon::Icon;
use rin_bevy_ui::icon_button::{IconButton, ToggleState};
use rin_bevy_ui::kbd::Kbd;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::link::{Link, LinkState};
use rin_bevy_ui::list::{CornerStyle, ItemNote, ItemView, List, ListLayout};
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value};
use rin_bevy_ui::panel::Panel;
use rin_bevy_ui::pill::Pill;
use rin_bevy_ui::push_button::{ButtonSize, PushButton};
use rin_bevy_ui::scroll_area::{ScrollArea, ScrollBar, ScrollContents};
use rin_bevy_ui::segmented_control::{Segment, SegmentedControl};
use rin_bevy_ui::setting_row::{SettingRow, SettingRows};
use rin_bevy_ui::stack::Stack;
use rin_bevy_ui::tab_bar::{TabBar, TabBarTab};
use rin_bevy_ui::text_field::{TextField, TextFieldRefs};
use rin_bevy_ui::theme::{Theme, ThemeColors};
use rin_bevy_ui::toolbar::{Toolbar, ToolbarButton, ToolbarLabels};
use rin_bevy_ui::tooltip::Tooltip;

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

#[test]
fn the_widget_components_are_in_the_type_registry() {
    with_registry(|registry| {
        assert_component::<PushButton>(registry);
        assert_component::<ButtonSize>(registry);
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
        assert_component::<ColorPalette>(registry);
        assert_component::<ColorPaletteCombo>(registry);
        assert_component::<ColorPushButton>(registry);
        assert_component::<ColorSquare>(registry);
        assert_component::<NumericStepper>(registry);
        assert_component::<ScrollArea>(registry);
        assert_component::<ScrollContents>(registry);
        assert_component::<ScrollBar>(registry);
        assert_component::<Stack>(registry);
        assert_component::<Card>(registry);
        assert_component::<Divider>(registry);
        assert_component::<Icon>(registry);
        assert_component::<SegmentedControl>(registry);
        assert_component::<SettingRow>(registry);
        assert_component::<SettingRows>(registry);
        assert_component::<Segment>(registry);
        assert_component::<TabBar>(registry);
        assert_component::<TabBarTab>(registry);
        assert_component::<IconButton>(registry);
        assert_component::<Panel>(registry);
        assert_component::<ToggleState>(registry);
        assert_component::<Kbd>(registry);
        assert_component::<Pill>(registry);
        assert_component::<Badge>(registry);
        assert_component::<FilterChips>(registry);
        assert_component::<ChipView>(registry);
        assert_component::<Gallery>(registry);
        assert_component::<GalleryCardView>(registry);
        assert_component::<Toolbar>(registry);
        assert_component::<ToolbarButton>(registry);
        assert_component::<ToolbarLabels>(registry);
        assert_component::<Tooltip>(registry);
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
        assert_component::<LinkState>(registry);
        assert_component::<color_palette_combo::State>(registry);
        assert_component::<Value>(registry);
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
fn a_widget_is_found_by_its_short_type_path() {
    with_registry(|registry| {
        let found = registry
            .get_with_short_type_path("PushButton")
            .expect("PushButton resolves by short type path");
        assert_eq!(found.type_id(), TypeId::of::<PushButton>());
    });
}
