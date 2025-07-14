use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use bevy_egui::EguiPlugin;
use smallvec::smallvec;
use ui::WidgetsPlugin;
use ui::color_palette::ColorPalette;
use ui::color_palette::MyColor;
use ui::list::CornerStyle;
use ui::list::List;
use ui::list::ListLayout;
use ui::push_button::PushButton;
use ui::select::Item;
use ui::select::Items;
use ui::select::Selection;
use ui::tailwind::COLORS;
use ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins((EguiPlugin {
            enable_multipass_for_primary_context: false,
        },))
        // .add_plugins(WorldInspectorPlugin::new())
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_observer(on_press)
        .add_systems(Update, update_theme.run_if(resource_changed::<Theme>))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands, theme: Res<Theme>) {
    commands.spawn(Camera2d);

    // Root container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::SpaceEvenly,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        BackgroundColor(theme.colors.background_primary),
        Root,
        children![
            (
                List,
                Selection {
                    items: smallvec![],
                    multi: true,
                },
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                List,
                ListLayout::Horizontal,
                Selection {
                    items: smallvec![],
                    multi: true,
                },
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                List,
                ListLayout::VerticalGrid { columns: 3 },
                Selection {
                    items: smallvec![],
                    multi: false,
                },
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                List,
                CornerStyle::Tabs,
                Selection {
                    items: smallvec![],
                    multi: false,
                },
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                List,
                ListLayout::Horizontal,
                CornerStyle::Tabs,
                Selection {
                    items: smallvec![],
                    multi: true,
                },
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                List,
                ListLayout::VerticalGrid { columns: 3 },
                CornerStyle::Tabs,
                Selection {
                    items: smallvec![],
                    multi: true,
                },
                related!(Items[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            ),
            (
                ColorPalette,
                ListLayout::VerticalGrid { columns: 11 },
                Items::spawn(SpawnIter(color_palette().into_iter()))
            ),
            (PushButton::from_label("Change theme"), ThemeButton),
        ],
    ));
}

fn color_palette() -> Vec<impl Bundle> {
    COLORS
        .iter()
        .map(|c| {
            (
                Item::new(format!("{:?}{:?}", c.class, c.variant)),
                MyColor(c.color),
            )
        })
        .collect()
}

#[derive(Component)]
struct ThemeButton;

fn on_press(
    trigger: Trigger<Pointer<Released>>,
    button: Query<(), With<ThemeButton>>,
    mut theme: ResMut<Theme>,
) {
    if button.get(trigger.target()).is_ok() {
        theme.light_dark_toggle();
    }
}

#[derive(Component)]
struct Root;

#[derive(Component)]
struct HelloWorldLabel;

fn update_theme(
    theme: Res<Theme>,
    mut background: Query<&mut BackgroundColor, With<Root>>,
    mut hello_world: Query<&mut TextColor, With<HelloWorldLabel>>,
) {
    for mut background_color in &mut background {
        background_color.0 = theme.colors.background_primary;
    }
    for mut text_color in &mut hello_world {
        text_color.0 = theme.colors.text_primary;
    }
}
