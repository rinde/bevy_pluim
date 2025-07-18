use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::spawn::SpawnWith;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use bevy_egui::EguiPlugin;
use smallvec::smallvec;
use ui::WidgetsPlugin;
use ui::color_palette::ColorPalette;
use ui::color_palette_combo::ColorPaletteCombo;
use ui::list::CornerStyle;
use ui::list::List;
use ui::list::ListLayout;
use ui::push_button::PushButton;
use ui::scroll_area::ScrollArea;
use ui::select::Item;
use ui::select::ItemValues;
use ui::select::Selection;
use ui::select::SelectionValue;
use ui::stack::Stack;
use ui::stack::StackChildSpawner;
use ui::stack::StackItem;
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
            display: Display::Grid,
            grid_template_columns: vec![
                GridTrack::max_content(),
                GridTrack::auto(),
                GridTrack::max_content(),
            ],
            grid_template_rows: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(theme.colors.background_primary),
        Root,
        Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
            let id = parent
                .spawn((
                    List,
                    CornerStyle::Tabs,
                    ListLayout::Horizontal,
                    SelectionValue {
                        items: smallvec![],
                        multi: false,
                    }
                    .into_component(),
                    related!(ItemValues[
                        (Item::new("Panel A"), StackItem::new(&create_a)),
                        (Item::new("Panel B"), StackItem::new(&create_b)),
                    ]),
                ))
                .id();
            // filler
            parent.spawn(Node {
                max_width: Val::Percent(100.0),
                ..default()
            });

            parent.spawn((PushButton::from_label("Toggle theme"), ThemeButton));

            parent.spawn((
                Node {
                    grid_column: GridPlacement::span(3),
                    display: Display::Flex,
                    ..default()
                },
                Stack { selection_ref: id },
            ));
        })),
    ));
}

fn create_a(spawner: StackChildSpawner) -> Entity {
    spawner.spawn((children![
        Text::new("Panel A"),
        (
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            ItemValues::spawn(SpawnIter(color_palette().into_iter()))
        ),
        (
            ColorPalette,
            ListLayout::VerticalGrid { columns: 11 },
            ItemValues::spawn(SpawnIter(color_palette().into_iter()))
        ),
        (
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            ItemValues::spawn(SpawnIter(color_palette().into_iter()))
        ),
    ],))
}

fn create_b(spawner: StackChildSpawner) -> Entity {
    spawner.spawn((children![
        Text::new("Panel B"),
        ScrollArea::bundle(
            |parent| {
                parent.spawn((
                    List,
                    Selection::default(),
                    ItemValues::spawn(SpawnIter(items())),
                ));
            },
            true,
            false,
        ),
        (
            Node {
                align_self: AlignSelf::Start,
                ..default()
            },
            children![(
                List,
                Selection::default(),
                related!(ItemValues[
                    Item::new("Netherlands"),
                    Item::new("Canada"),
                    Item::new("Indonesia"),
                    Item::new("Portugal"),
                    Item::new("Belgium")
                ]),
            )]
        ),
    ],))
}
fn color_palette() -> Vec<impl Bundle> {
    COLORS.iter().map(|c| (Item { data: c.color })).collect()
}

fn items() -> impl Iterator<Item = Item<String>> {
    (1..100).map(|i| Item::new(i.to_string()))
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
