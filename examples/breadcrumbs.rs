//! `BreadCrumbs` and a `Stack` sharing one selection, with reused `StackItem` creators.

use std::sync::atomic::{AtomicBool, Ordering};

use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy_pluim::WidgetsPlugin;
use bevy_pluim::breadcrumbs::BreadCrumbs;
use bevy_pluim::label::Label;
use bevy_pluim::push_button::PushButton;
use bevy_pluim::select::{Item, ItemParam, Selection, SelectionParam, SelectionValue};
use bevy_pluim::stack::{Stack, StackChildSpawner, StackItem};
use bevy_pluim::theme::Theme;

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

#[derive(Component, Copy, Clone, Debug, Default)]
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
    // Root container
    commands.spawn_scene(bsn! {
        Node {
            width: percent(100.0),
            height: percent(100.0),
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Start,
            flex_direction: FlexDirection::Row,
            flex_wrap: FlexWrap::Wrap,
        }
        Children [
            Node {
                width: percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
            }
            Children [
                ~{BreadCrumbs}
                #crumbs
                BreadCrumbsSelection
                @{Selection::builder().one_or_more().scene_with_items(items)}
                --
                ~{PushButton::from_label("Change theme")}
                on(|_: On<Activate>, mut theme: ResMut<Theme>| theme.light_dark_toggle())
            ]
            --
            Node {
                width: percent(100.0),
                height: percent(100.0),
                justify_content: JustifyContent::Start,
                align_items: AlignItems::Center,
            }
            Children [ Stack { selection_ref: #crumbs } ]
        ]
    });
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

/// An observer that pushes `target` onto the trail.
fn go_to(
    target: BreadCrumbsAction,
) -> impl Fn(
    On<Activate>,
    SelectionParam<With<BreadCrumbsSelection>>,
    ItemParam<String, &BreadCrumbsAction>,
) + Clone {
    move |_, mut selections, items| push_to_selection(target, &mut selections, items)
}

fn pop_on_activate(_: On<Activate>, mut selections: SelectionParam<With<BreadCrumbsSelection>>) {
    selections.update_selection_single(SelectionValue::pop);
}

fn create_home(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
        Children [
            ~{Label::header("Home")}
            --
            ~{PushButton::from_label("Go to Panel A")}
            on(go_to(BreadCrumbsAction::PanelA))
            --
            ~{PushButton::from_label("Go to Panel A variant")}
            on(go_to(BreadCrumbsAction::PanelAvariant))
            --
            ~{PushButton::from_label("Go to Panel B")}
            on(go_to(BreadCrumbsAction::PanelB))
        ]
        },
    );
}

static INITIALIZED_A: AtomicBool = AtomicBool::new(false);

fn create_a(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let initialized = INITIALIZED_A.load(Ordering::Relaxed);
    assert!(!initialized);
    INITIALIZED_A.store(true, Ordering::Relaxed);
    spawner.spawn_scene(
        &mut commands,
        bsn! {
        Children [
            ~{Label::header("Panel A")}
            --
            ~{PushButton::from_label("Go back to home")}
            on(pop_on_activate)
            --
            ~{PushButton::from_label("Go to panel A variant")}
            on(|_: On<Activate>,
                mut selections: SelectionParam<With<BreadCrumbsSelection>>,
                items: ItemParam<String, &BreadCrumbsAction>| {
                let Some(item_a_ref) =
                    items.find_item_ref(|_, bca| **bca == BreadCrumbsAction::PanelAvariant)
                else {
                    return;
                };
                selections.update_selection_single(|x| x.pop()?.push(item_a_ref));
            })
        ]
        },
    );
}

fn create_b(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
        Children [
            ~{Label::header("Panel B")}
            --
            ~{PushButton::from_label("Go back to home")}
            on(pop_on_activate)
            --
            ~{PushButton::from_label("Go to panel B sub")}
            on(go_to(BreadCrumbsAction::PanelBsub))
        ]
        },
    );
}

fn create_b_sub(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
        Children [
            ~{Label::header("Panel B sub")}
            --
            ~{PushButton::from_label("Go back to panel B")}
            on(pop_on_activate)
        ]
        },
    );
}
