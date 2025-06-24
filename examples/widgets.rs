use bevy::prelude::*;
use ui::colors::BG_SECONDARY;
use ui::colors::TEXT_PRIMARY;
use ui::combobox::Combo;
use ui::combobox::ComboBoxConfig;
use ui::combobox::ComboBoxPlugin;
use ui::combobox::Widgets;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(UiDebugOptions {
            enabled: false,
            // `UiDebugOptions` has a few new options, but for now we'll leave the defaults.
            ..default()
        })
        .add_plugins(ComboBoxPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, events)
        .run();
}

fn setup(mut commands: Commands, mut widgets: ResMut<Widgets>) {
    // Camera
    commands.spawn(Camera2d);

    // Root container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            ..default()
        },
        BackgroundColor(BG_SECONDARY),
        children![
            (
                Text("Select Component".to_string()),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(TEXT_PRIMARY),
            ),
            // spacing
            Node {
                height: Val::Px(40.0),
                ..default()
            },
            (
                CountryComboBox,
                widgets.combobox(ComboBoxConfig {
                    label: Some("Country".to_string()),
                    options: vec![
                        "NL".to_string(),
                        "PT".to_string(),
                        "ID".to_string(),
                        "CA".to_string(),
                    ],
                    ..default()
                })
            ),
            (
                ContinentComboBox,
                widgets.combobox(ComboBoxConfig {
                    label: Some("Continent".to_string()),
                    options: vec![
                        "Europe".to_string(),
                        "Asia".to_string(),
                        "North-America".to_string(),
                    ],
                    ..default()
                })
            )
        ],
    ));
}

#[derive(Component)]
struct CountryComboBox;

#[derive(Component)]
struct ContinentComboBox;

fn events(country_query: Query<&Combo, (With<CountryComboBox>, Changed<Combo>)>) {
    for x in country_query {
        println!("Country selected: {:?}", x.selected_text());
    }
}
