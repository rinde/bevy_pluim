use std::sync::atomic::{AtomicBool, Ordering};

use bevy::ecs::spawn::SpawnWith;
use bevy::prelude::*;
use bevy_egui::EguiPlugin;
use rin_bevy_ui::WidgetsPlugin;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::label::Label;
use rin_bevy_ui::push_button::PushButton;
use rin_bevy_ui::select::{Item, ItemParam, Selection, SelectionParam, SelectionValue};
use rin_bevy_ui::stack::{Stack, StackChildSpawner, StackItem};
use rin_bevy_ui::theme::Theme;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(WidgetsPlugin)
        .add_plugins(EguiPlugin::default())
        // .add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new())
        .insert_resource(UiDebugOptions {
            enabled: false,
            line_width: 1.0,
            show_hidden: false,
            show_clipped: false,
        })
        .add_observer(on_press)
        .add_systems(Startup, setup)
        .run();
}

#[derive(Component, Copy, Clone, Debug)]
struct BreadCrumbsSelection;

#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
enum BreadCrumbsAction {
    Home,
    PanelA,
    PanelAvariant,
    PanelB,
    PanelBsub,
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    let items = [
        (
            BreadCrumbsAction::Home,
            Item::new("Home"),
            StackItem::new(commands.register_system(create_home)),
        ),
        (
            BreadCrumbsAction::PanelA,
            Item::new("Panel A"),
            StackItem::new_with_reference(0, commands.register_system(create_a)),
        ),
        (
            BreadCrumbsAction::PanelAvariant,
            Item::new("Panel A variant"),
            StackItem::new_reference(0),
        ),
        (
            BreadCrumbsAction::PanelB,
            Item::new("Panel B"),
            StackItem::new(commands.register_system(create_b)),
        ),
        (
            BreadCrumbsAction::PanelBsub,
            Item::new("Panel B sub"),
            StackItem::new(commands.register_system(create_b_sub)),
        ),
    ];
    let selection_ref = commands
        .spawn((
            BreadCrumbsSelection,
            Selection::builder().one_or_more().build_with_items(items),
        ))
        .id();

    // Root container
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
            ..default()
        },
        children![
            (
                Node {
                    width: Val::Percent(100.0),
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                children![
                    (BreadCrumbs, Selection::reference(selection_ref)),
                    (PushButton::from_label("Change theme"), ThemeButton),
                ]
            ),
            (
                Node {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Start,
                    align_items: AlignItems::Center,
                    ..default()
                },
                children![Stack { selection_ref }],
            )
        ],
    ));
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

fn push_to_selection(
    target: BreadCrumbsAction,
    selections: &mut SelectionParam<With<BreadCrumbsSelection>>,
    items: ItemParam<String, &BreadCrumbsAction>,
) {
    let Some(item_a_ref) = items.find_item_ref(|_, bca| **bca == target) else {
        return;
    };
    selections.update_selection_single(|selection| selection.push(item_a_ref));
}

fn pop_on_pointer_release(
    _: Trigger<Pointer<Released>>,
    mut selections: SelectionParam<With<BreadCrumbsSelection>>,
) {
    selections.update_selection_single(SelectionValue::pop);
}

fn create_home(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Home"));

                parent
                    .spawn(PushButton::from_label("Go to Panel A"))
                    .observe(
                        |_: Trigger<Pointer<Released>>,
                         mut selections: SelectionParam<With<BreadCrumbsSelection>>,
                         items: ItemParam<String, &BreadCrumbsAction>| {
                            push_to_selection(BreadCrumbsAction::PanelA, &mut selections, items);
                        },
                    );

                parent
                    .spawn(PushButton::from_label("Go to Panel A variant"))
                    .observe(
                        |_: Trigger<Pointer<Released>>,
                         mut selections: SelectionParam<With<BreadCrumbsSelection>>,
                         items: ItemParam<String, &BreadCrumbsAction>| {
                            push_to_selection(
                                BreadCrumbsAction::PanelAvariant,
                                &mut selections,
                                items,
                            );
                        },
                    );

                parent
                    .spawn(PushButton::from_label("Go to Panel B"))
                    .observe(
                        |_: Trigger<Pointer<Released>>,
                         mut selections: SelectionParam<With<BreadCrumbsSelection>>,
                         items: ItemParam<String, &BreadCrumbsAction>| {
                            push_to_selection(BreadCrumbsAction::PanelB, &mut selections, items);
                        },
                    );
            })),
        ),
    );
}

static INITIALIZED_A: AtomicBool = AtomicBool::new(false);

fn create_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let initialized = INITIALIZED_A.load(Ordering::Relaxed);
    assert!(!initialized);
    INITIALIZED_A.store(true, Ordering::Relaxed);
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Panel A"));

                parent
                    .spawn(PushButton::from_label("Go back to home"))
                    .observe(pop_on_pointer_release);

                parent
                    .spawn(PushButton::from_label("Go to panel A variant"))
                    .observe(
                        |_: Trigger<Pointer<Released>>,
                         mut selections: SelectionParam<With<BreadCrumbsSelection>>,
                         items: ItemParam<String, &BreadCrumbsAction>| {
                            let Some(item_a_ref) = items
                                .find_item_ref(|_, bca| **bca == BreadCrumbsAction::PanelAvariant)
                            else {
                                return;
                            };
                            selections.update_selection_single(|x| x.pop()?.push(item_a_ref));
                        },
                    );
            })),
        ),
    );
}
fn create_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Panel B"));

                parent
                    .spawn(PushButton::from_label("Go back to home"))
                    .observe(pop_on_pointer_release);
                parent
                    .spawn(PushButton::from_label("Go to panel B sub"))
                    .observe(
                        |_: Trigger<Pointer<Released>>,
                         mut selections: SelectionParam<With<BreadCrumbsSelection>>,
                         items: ItemParam<String, &BreadCrumbsAction>| {
                            push_to_selection(BreadCrumbsAction::PanelBsub, &mut selections, items);
                        },
                    );
            })),
        ),
    );
}
fn create_b_sub(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn(
        &mut commands,
        (
            Node::DEFAULT,
            Children::spawn(SpawnWith(|parent: &mut ChildSpawner| {
                parent.spawn(Label::header("Panel B sub"));

                parent
                    .spawn(PushButton::from_label("Go back to panel B"))
                    .observe(pop_on_pointer_release);
            })),
        ),
    );
}
