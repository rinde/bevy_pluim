//! `SettingRow`: rows of a title over a description with a control on the right, in a
//! `SettingRows` column.

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value};
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::segmented_control::SegmentedControl;
use rin_bevy_ui::select::{Item, Selection};
use rin_bevy_ui::setting_row::{SettingRow, SettingRowRefs, SettingRows};
use rin_bevy_ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .insert_resource(GlobalUiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
            ..default()
        })
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let root = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        })
        .id();
    let rows = commands
        .spawn((
            SettingRows,
            Node {
                width: Val::Px(420.0),
                ..default()
            },
            ChildOf(root),
        ))
        .id();
    let appearance = commands
        .spawn((
            SettingRow::new("Appearance", "Light or dark"),
            ChildOf(rows),
        ))
        .id();
    let text_size = commands
        .spawn((SettingRow::new("Text size", "In points"), ChildOf(rows)))
        .id();
    let reset = commands
        .spawn((
            SettingRow::new("Reset", "Back to the defaults"),
            ChildOf(rows),
        ))
        .id();
    commands
        .spawn((PushButton::from_label("Change theme"), ChildOf(root)))
        .observe(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle());
    // The rows name their control slots once spawned; the app fills them.
    commands.queue(move |world: &mut World| {
        world.flush();
        let slot = |world: &World, row: Entity| {
            world
                .get::<SettingRowRefs>(row)
                .map(|refs| refs.control_slot)
        };
        if let Some(slot) = slot(world, appearance) {
            world.spawn((
                SegmentedControl,
                Selection::builder()
                    .one()
                    .build_with_items(["Light", "Dark"].map(Item::new)),
                ChildOf(slot),
            ));
        }
        if let Some(slot) = slot(world, text_size) {
            world.spawn((
                NumericStepper::default(),
                Value::new_u8(12, 8, 20, 1),
                ChildOf(slot),
            ));
        }
        if let Some(slot) = slot(world, reset) {
            world.spawn((PushButton::from_label("Reset"), ChildOf(slot)));
        }
    });
}
