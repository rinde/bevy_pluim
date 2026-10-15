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
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            SettingRows
            Node { width: px(420.0) }
            Children [
                ~{SettingRow::new("Appearance", "Light or dark")} Control::Appearance
                --
                ~{SettingRow::new("Text size", "In points")} Control::TextSize
                --
                ~{SettingRow::new("Reset", "Back to the defaults")} Control::Reset
            ]
            --
            ~{PushButton::from_label("Change theme")}
            on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
        ]
    });
    // The rows name their control slots once spawned; the app fills them.
    commands.queue(|world: &mut World| {
        world.flush();
        let rows: Vec<(Entity, Control)> = world
            .query::<(&SettingRowRefs, &Control)>()
            .iter(world)
            .map(|(refs, control)| (refs.control_slot, *control))
            .collect();
        for (slot, control) in rows {
            match control {
                Control::Appearance => {
                    world.spawn((
                        SegmentedControl,
                        Selection::builder()
                            .one()
                            .build_with_items(["Light", "Dark"].map(Item::new)),
                        ChildOf(slot),
                    ));
                }
                Control::TextSize => {
                    world.spawn((
                        NumericStepper::default(),
                        Value::new_u8(12, 8, 20, 1),
                        ChildOf(slot),
                    ));
                }
                Control::Reset => {
                    world.spawn((PushButton::from_label("Reset"), ChildOf(slot)));
                }
            }
        }
    });
}

/// The control a row gets in its slot.
#[derive(Component, Default, Clone, Copy)]
enum Control {
    #[default]
    Appearance,
    TextSize,
    Reset,
}
