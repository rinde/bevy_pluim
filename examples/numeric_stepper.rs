//! `NumericStepper`.

use bevy::prelude::*;
use bevy::ui_render::GlobalUiDebugOptions;
use bevy::ui_widgets::Activate;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::numeric_stepper::{NumericStepper, Value};
use rin_bevy_ui::push_button::PushButton;
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
        .add_observer(on_press)
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            row_gap: px(10.0),
            column_gap: px(10.0),
            padding: UiRect::all(px(10.0)),
        }
        Children [
            inserted(NumericStepper::default())
            --
            inserted(NumericStepper::default())
            ~{Value::new_u16(10, 5, 15, 1)}
            --
            inserted(NumericStepper::default())
            ~{Value::new_u32(10, 5, 15, 1)}
            --
            inserted(NumericStepper::default())
            ~{Value::new_f32(10.0, 5.0, 15.0, 0.5)}
            --
            inserted(NumericStepper {
                format: level,
                decrease_name: "Lower the level".into(),
                increase_name: "Raise the level".into(),
                ..default()
            })
            ~{Value::new_u8(0, 0, 56, 7)}
            --
            ~{PushButton::from_label("Change theme")} ThemeButton
        ]
    });
}

/// A `bsn!` entry that inserts `component` as it is.
///
/// `~{value}` takes only a value that is `Unpin`, which a `Handle` is not, so a `NumericStepper`, which
/// holds the `Handle`s of its icons,
/// goes in through this.
fn inserted<C: Component + Clone>(
    component: C,
) -> impl Template<Output = C> + Send + Sync + 'static {
    template(move |_| Ok(component.clone()))
}

/// "Ground" at 0 and "+7", "+14", ... above it.
fn level(value: &Value) -> String {
    match value.as_u8() {
        Some(0) | None => "Ground".to_string(),
        Some(n) => format!("+{n}"),
    }
}

#[derive(Component, Default, Clone)]
struct ThemeButton;

fn on_press(trigger: On<Activate>, button: Query<(), With<ThemeButton>>, mut theme: ResMut<Theme>) {
    if button.get(trigger.event().event_target()).is_ok() {
        theme.light_dark_toggle();
    }
}
