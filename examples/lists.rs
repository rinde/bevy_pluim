use bevy::DefaultPlugins;
use bevy::app::App;
use bevy::app::Startup;
use bevy::color::palettes::tailwind::GRAY_300;
use bevy::color::palettes::tailwind::GRAY_700;
use bevy::ecs::spawn::SpawnIter;
use bevy::ecs::system::Commands;
use bevy::prelude::*;
use bevy::ui::UiDebugOptions;
use bevy_egui::EguiPlugin;
use smallvec::smallvec;
use ui::WidgetsPlugin;
use ui::list::CornerStyle;
use ui::list::ItemView;
use ui::list::ItemViewFactory;
use ui::list::ItemsThemerInput;
use ui::list::List;
use ui::list::ListLayout;
use ui::push_button::PushButton;
use ui::select::Item;
use ui::select::Items;
use ui::select::Selection;
use ui::select::ViewOf;
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
                List,
                ListLayout::VerticalGrid { columns: 11 },
                Selection {
                    items: smallvec![],
                    multi: false,
                },
                ItemViewFactory {
                    item_creator: Box::new(|parent, world, item_id, _item| {
                        let color = world.entity(item_id).get::<MyColor>().unwrap();
                        parent
                            .spawn((
                                Node {
                                    border: UiRect::all(Val::Px(1.0)),
                                    width: Val::Px(35.0),
                                    height: Val::Px(35.0),
                                    ..default()
                                },
                                BorderRadius::all(Val::Px(5.0)),
                                BackgroundColor(color.0),
                            ))
                            .id()
                    }),
                    item_themer: &|reg| {
                        reg.register_item_theme_system(theme_system);
                    }
                },
                Items::spawn(SpawnIter(color_palette().into_iter()))
            ),
            (PushButton::from_label("Change theme"), ThemeButton),
        ],
    ));
}

fn theme_system(
    inp: In<ItemsThemerInput>,
    theme: Res<Theme>,
    mut item_views: Query<(
        &Children,
        &ItemView,
        &mut Node,
        &mut BackgroundColor,
        &mut BorderRadius,
        &mut BorderColor,
    )>,
    selections: Query<&Selection>,
    mut color_items: Query<(&mut Node, &mut BorderRadius, &mut BorderColor), Without<ItemView>>,
) {
    let ItemsThemerInput {
        children,
        view_of,
        list_id,
        layout: _,
        corner_style: _,
    } = inp.0;

    if children.is_empty() {
        println!("No items to theme");
        return;
    }
    // either this List is a view of some items defined elsewhere, or the items are
    // define on the List entity.
    let selection_id = view_of.map_or(list_id, |ViewOf(selection_id)| selection_id);
    let selection = selections.get(selection_id).unwrap();

    for item_view_id in &children {
        let (children, ItemView { item_id }, mut node, mut bg, mut br, mut _bc) =
            item_views.get_mut(*item_view_id).unwrap();
        *br = BorderRadius::all(Val::Px(5.0));
        let is_selected = selection.is_selected(*item_id);

        if is_selected {
            bg.0 = GRAY_700.into();
        } else {
            bg.0 = theme.colors.background_primary;
        }
        node.padding = UiRect::all(Val::Px(1.0));

        let (mut _color_node, mut border_radius, mut border_color) =
            color_items.get_mut(*children.first().unwrap()).unwrap();

        if is_selected {
            border_color.0 = GRAY_300.into(); //theme.colors.border_highlight;
        } else {
            border_color.0 = theme.colors.background_primary; //theme.colors.border;
        }
        // color_node.border = UiRect::all(theme.border_thickness);
        *border_radius = BorderRadius::all(Val::Px(5.0));
    }
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
struct MyColor(Color);

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
