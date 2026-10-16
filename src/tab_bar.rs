//! `TabBar`: a row of tabs, one per item of a [`Selection`], on `bevy_ui_widgets`' tab list.

use std::collections::HashMap;

use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{SelectedTab, Tab, TabList, ValueChange};

use crate::focus::{AccessibilitySync, sync_label};
use crate::segmented_control::segment_colors;
use crate::select::{Item, ItemsChanged, Selection, SelectionParam};
use crate::theme::Theme;

/// The height of a tab, in logical pixels.
const TAB_HEIGHT: f32 = 30.0;
/// The least width of a tab.
const TAB_MIN_WIDTH: f32 = 48.0;
/// The padding between a tab's text and its sides.
const TAB_SIDE_PADDING: f32 = 12.0;
/// The padding of the bar around its tabs.
const BAR_PADDING: f32 = 3.0;
/// The gap between two tabs.
const TAB_GAP: f32 = 2.0;
/// The size of a tab's text.
const TAB_FONT_SIZE: f32 = 13.0;

/// Registers the systems and observers of [`TabBar`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct TabBarPlugin;

impl Plugin for TabBarPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_tab_bars)
                    .run_if(resource_changed::<Theme>),
                selection_change,
                hover_change,
            ),
        )
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
        .add_observer(on_insert_selection)
        .add_observer(on_value_change)
        .register_type::<TabBar>()
        .register_type::<TabBarTab>();
    }
}

/// A row of tabs that picks one item, to switch between the panels of a
/// [`Stack`](crate::stack::Stack), say.
///
/// Give it a [`Selection`] (own, with its [`Item<String>`]s, or a
/// [`Selection::Ref`] to one that other widgets share) of the kind
/// [`SelectionValue::One`](crate::select::SelectionValue::One); it shows one [`TabBarTab`] per
/// item, with the selected one highlighted in the theme's `segmented_*` colours.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use rin_bevy_ui::tab_bar::TabBar;
/// # use rin_bevy_ui::select::{Item, Selection};
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         TabBar,
///         Selection::builder()
///             .one()
///             .build_with_items(["General", "Audio"].map(Item::new)),
///     ));
/// }
/// ```
///
/// The bar is a `bevy_ui_widgets` [`TabList`] and each tab a [`Tab`], so the bar is one tab
/// stop (the selected tab, or the focused one), the arrow keys, Home and End move the focus
/// over the tabs, and a click or Enter / Space on a focused tab selects it. The tab list asks
/// for the change, and the [`Selection`] decides: [`SelectedTab`] and Bevy's `Selected` on the
/// tabs follow the selection, also when it is changed from elsewhere.
///
/// To a screen reader the bar is a tab list and each tab a tab named after its item's text,
/// selected while its item is. The library gives the bar no label; an app can insert Bevy's
/// `AccessibleLabel` on it.
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Name::new("TabBar"),
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        align_self: AlignSelf::FlexStart,
        padding: UiRect::all(Val::Px(BAR_PADDING)),
        column_gap: Val::Px(TAB_GAP),
        ..default()
    },
    BackgroundColor::default(),
    TabList::default(),
)]
pub struct TabBar;

impl TabBar {
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_tab_bars, Some(context.entity));
    }
}

/// One tab of a [`TabBar`]: a [`Tab`] showing one item.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Name::new("TabBarTab"), Tab, Hovered, BackgroundColor::default())]
pub struct TabBarTab {
    /// The item this tab shows and selects.
    pub item_ref: Entity,
    /// The bar this tab belongs to.
    pub bar_ref: Entity,
}

/// Marks the text of a [`TabBarTab`].
#[derive(Component, Debug, Copy, Clone)]
struct TabBarTabLabel;

/// Selects the item of the tab the tab list asks for.
fn on_value_change(
    change: On<ValueChange<Option<Entity>>>,
    bars: Query<(), With<TabBar>>,
    tabs: Query<&TabBarTab>,
    mut selections: SelectionParam,
) {
    let bar = change.event().source;
    if !bars.contains(bar) {
        return;
    }
    let Some(tab) = change.event().value else {
        return;
    };
    let Ok(tab) = tabs.get(tab) else {
        return;
    };
    if tab.bar_ref != bar {
        return;
    }
    selections.update_selection(bar, |selection| selection.set_selection([tab.item_ref]));
}

fn theme_tab_bars(
    bar_id: In<Option<Entity>>,
    theme: Res<Theme>,
    selections: SelectionParam,
    mut bars: Query<(Entity, &mut Node, &mut BackgroundColor), (With<TabBar>, Without<TabBarTab>)>,
    mut tabs: Query<(&TabBarTab, &Hovered, &mut Node, &mut BackgroundColor), Without<TabBar>>,
    mut labels: Query<(&ChildOf, &mut TextColor, &mut TextFont), With<TabBarTabLabel>>,
    states: Query<(&TabBarTab, &Hovered)>,
) {
    for (bar, mut node, mut background) in &mut bars {
        if bar_id.0.is_some_and(|id| id != bar) {
            continue;
        }
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = theme.colors.segmented_track;
    }

    for (tab, hovered, mut node, mut background) in &mut tabs {
        if bar_id.0.is_some_and(|id| id != tab.bar_ref) {
            continue;
        }
        let selected = selections
            .get(tab.bar_ref)
            .is_some_and(|selection| selection.is_selected(tab.item_ref));
        node.height = Val::Px(TAB_HEIGHT);
        node.min_width = Val::Px(TAB_MIN_WIDTH);
        node.padding = UiRect::axes(Val::Px(TAB_SIDE_PADDING), Val::ZERO);
        node.align_items = AlignItems::Center;
        node.justify_content = JustifyContent::Center;
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = segment_colors(&theme.colors, selected, hovered.get()).0;
    }

    for (child_of, mut text_color, mut text_font) in &mut labels {
        let Ok((tab, hovered)) = states.get(child_of.parent()) else {
            continue;
        };
        if bar_id.0.is_some_and(|id| id != tab.bar_ref) {
            continue;
        }
        let selected = selections
            .get(tab.bar_ref)
            .is_some_and(|selection| selection.is_selected(tab.item_ref));
        text_color.0 = segment_colors(&theme.colors, selected, hovered.get()).1;
        *text_font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(TAB_FONT_SIZE),
            weight: FontWeight::SEMIBOLD,
            ..default()
        };
    }
}

/// Names each tab after its item's text. The roles are `bevy_ui_widgets`' own, and the
/// selected state follows Bevy's `Selected`, which the tab list derives from [`SelectedTab`].
fn sync_accessibility(
    mut tabs: Query<(&TabBarTab, &mut AccessibilityNode)>,
    items: Query<&Item<String>>,
) {
    for (tab, mut node) in &mut tabs {
        if let Ok(item) = items.get(tab.item_ref) {
            sync_label(&mut node, &item.data);
        }
    }
}

/// Restyles the tabs of a bar when a tab gains or loses the pointer.
fn hover_change(tabs: Query<&TabBarTab, Changed<Hovered>>, mut commands: Commands) {
    for tab in &tabs {
        commands.run_system_cached_with(theme_tab_bars, Some(tab.bar_ref));
    }
}

/// Points the [`SelectedTab`] of `bar` at the tab of its selected item, or at none.
fn sync_selected_tab(
    bar: In<Entity>,
    bars: Query<(Option<&Children>, &SelectedTab), With<TabBar>>,
    tabs: Query<&TabBarTab>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let Ok((children, selected_tab)) = bars.get(*bar) else {
        return;
    };
    let Some(selection) = selections.get(*bar) else {
        return;
    };
    let wanted = children
        .into_iter()
        .flat_map(RelationshipTarget::iter)
        .find(|child| {
            tabs.get(*child)
                .is_ok_and(|tab| selection.is_selected(tab.item_ref))
        });
    if selected_tab.0 != wanted {
        commands.entity(*bar).insert(SelectedTab(wanted));
    }
}

/// Keeps [`SelectedTab`] and the colours of the tabs in step with the selection, own or
/// referenced.
fn selection_change(
    changed: Query<(), Changed<Selection>>,
    bars: Query<Entity, With<TabBar>>,
    mut commands: Commands,
) {
    if changed.is_empty() {
        return;
    }
    for bar in &bars {
        commands.run_system_cached_with(sync_selected_tab, bar);
        commands.run_system_cached_with(theme_tab_bars, Some(bar));
    }
}

struct ItemsChangedInput {
    bar_ref: Entity,
    items: Vec<Entity>,
}

fn on_items_changed(
    input: In<ItemsChangedInput>,
    items_query: Query<&Item<String>>,
    bars: Query<Option<&Children>, With<TabBar>>,
    tabs: Query<(Entity, &TabBarTab)>,
    mut commands: Commands,
) {
    let ItemsChangedInput { bar_ref, items } = &*input;
    let Ok(children) = bars.get(*bar_ref) else {
        return;
    };

    let mut missing = items.clone();
    for (tab_id, tab) in children
        .into_iter()
        .flat_map(RelationshipTarget::iter)
        .filter_map(|c| tabs.get(c).ok())
    {
        let before = missing.len();
        missing.retain(|item| *item != tab.item_ref);
        if before == missing.len() {
            // a tab without a corresponding item
            commands.entity(tab_id).despawn();
        }
    }

    for item_ref in missing {
        let Ok(item) = items_query.get(item_ref) else {
            continue;
        };
        commands.entity(*bar_ref).with_children(|spawner| {
            spawner
                .spawn((
                    TabBarTab {
                        item_ref,
                        bar_ref: *bar_ref,
                    },
                    Node::default(),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(item.data.clone()),
                        TabBarTabLabel,
                        Pickable::IGNORE,
                    ));
                });
        });
    }
    commands.run_system_cached_with(sync_selected_tab, *bar_ref);
    commands.run_system_cached_with(theme_tab_bars, Some(*bar_ref));
}

fn on_insert_selection(
    t: On<Insert<Selection>>,
    bars: Query<(), With<TabBar>>,
    selections: SelectionParam,
    mut commands: Commands,
    mut existing_values_ref_map: Local<HashMap<Entity, Entity>>,
) {
    let bar_ref = t.event().event_target();
    if bars.get(bar_ref).is_err() {
        return;
    }
    let Some((values_ref, all_items)) = selections.get_item_values(bar_ref) else {
        return;
    };
    if existing_values_ref_map.get(&bar_ref) == Some(&values_ref) {
        return;
    }
    existing_values_ref_map.insert(bar_ref, values_ref);

    commands
        .entity(values_ref)
        .observe(move |t: On<ItemsChanged>, mut commands: Commands| {
            commands.run_system_cached_with(
                on_items_changed,
                ItemsChangedInput {
                    bar_ref,
                    items: t.event().items.iter().collect(),
                },
            );
        });

    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            bar_ref,
            items: all_items.iter().collect(),
        },
    );
}
