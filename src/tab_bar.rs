//! `TabBar`: a row of tabs, one per item of a [`Selection`], on `bevy_ui_widgets`' tab list.

use std::sync::Arc;

use bevy::a11y::AccessibilityNode;
use bevy::ecs::entity::EntityHashSet;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::input_focus::{FocusedInput, InputFocus, InputFocusSystems};
use bevy::picking::hover::Hovered;
use bevy::platform::collections::HashSet;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{SelectedTab, Tab, TabList, ValueChange};

use crate::focus::{AccessibilitySync, HiddenDirty, collect_hidden_changes, is_hidden, sync_label};
use crate::indicator::{self, Indicator};
use crate::segmented_control::segment_colors;
use crate::select::{
    Item, ItemsObserver, ItemsRef, Selection, SelectionParam, items_in_order,
    unwatch_dangling_items, watch_items,
};
use crate::theme::Theme;

const TAB_HEIGHT: f32 = 30.0;
const TAB_MIN_WIDTH: f32 = 48.0;
const TAB_SIDE_PADDING: f32 = 12.0;
const BAR_PADDING: f32 = 3.0;
const TAB_GAP: f32 = 2.0;
const TAB_FONT_SIZE: f32 = 13.0;

/// Registers the systems and observers of [`TabBar`].
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
                clear_dangling_tab_bars,
                selection_change,
                hover_change,
                sync_label_text,
                (
                    indicator::spawn::<TabIndicator, TabBar>,
                    indicator::animate::<TabIndicator, TabBar, TabBarTab>,
                )
                    .chain(),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                sync_accessibility.in_set(AccessibilitySync),
                // Run after the tab list's roving `TabIndex`, so a hidden bar's -1 wins.
                sync_tab_index
                    .after(collect_hidden_changes)
                    .after(InputFocusSystems::FocusChangeEvents),
            ),
        )
        .add_observer(on_insert_selection)
        .add_observer(on_value_change)
        .add_observer(keep_navigation_keys)
        .register_type::<TabBar>()
        .register_type::<TabBarTab>();
    }
}

/// A row of tabs that picks one item.
///
/// Give it a [`Selection`] of [`SelectionValue::One`](crate::select::SelectionValue::One)
/// over [`Item<String>`]s; it shows one [`TabBarTab`] per item.
///
/// ```no_run
/// # use bevy::prelude::*;
/// # use bevy_pluim::tab_bar::TabBar;
/// # use bevy_pluim::select::{Item, Selection};
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
/// The bar is a [`TabList`] and each tab a [`Tab`]; the [`Selection`] decides the selected tab.
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

/// The highlight behind the selected tab of a [`TabBar`].
#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("TabIndicator"), Node, BackgroundColor::default())]
struct TabIndicator {
    owner: Entity,
}

impl Indicator for TabIndicator {
    fn of(owner: Entity) -> Self {
        Self { owner }
    }

    fn owner(&self) -> Entity {
        self.owner
    }
}

/// One tab of a [`TabBar`]: a [`Tab`] showing one item.
///
/// Read its fields with [`item`](Self::item) and [`owner`](Self::owner).
///
/// ```compile_fail
/// use bevy_pluim::tab_bar::TabBarTab;
/// # let (item, bar) = (bevy::prelude::Entity::PLACEHOLDER, bevy::prelude::Entity::PLACEHOLDER);
/// let _ = TabBarTab { item, owner: bar };
/// ```
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::tab_bar::TabBarTab;
/// fn change(mut tabs: Query<&mut TabBarTab>) {}
/// bevy::ecs::system::assert_is_system(change);
/// ```
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(Name::new("TabBarTab"), Tab, Hovered, BackgroundColor::default())]
pub struct TabBarTab {
    item: Entity,
    owner: Entity,
}

impl TabBarTab {
    pub(crate) const fn new(item: Entity, owner: Entity) -> Self {
        Self { item, owner }
    }

    /// The item this tab shows and selects.
    #[must_use]
    pub const fn item(&self) -> Entity {
        self.item
    }

    /// The [`TabBar`] this tab belongs to.
    #[must_use]
    pub const fn owner(&self) -> Entity {
        self.owner
    }
}

#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
struct TabBarTabLabel;

/// Keeps left, right, Home and End on a focused tab from reaching directional navigation.
///
/// The tab list stops only the first press, so this stops repeats and releases too.
fn keep_navigation_keys(
    mut input: On<FocusedInput<KeyboardInput>>,
    tabs: Query<(), With<TabBarTab>>,
    focus: Res<InputFocus>,
) {
    let tab = input.event_target();
    if focus.get() != Some(tab) || !tabs.contains(tab) {
        return;
    }
    if matches!(
        input.event().input.key_code,
        KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::Home | KeyCode::End
    ) {
        input.propagate(false);
    }
}

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
    if tab.owner() != bar {
        return;
    }
    selections.update_selection(bar, |selection| selection.set_selection([tab.item()]));
}

fn theme_tab_bars(
    bar_id: In<Option<Entity>>,
    theme: Res<Theme>,
    selections: SelectionParam,
    mut bars: Query<
        (Entity, &mut Node, &mut BackgroundColor),
        (With<TabBar>, Without<TabBarTab>, Without<TabIndicator>),
    >,
    mut tabs: Query<
        (&TabBarTab, &mut Node, &mut BackgroundColor),
        (Without<TabBar>, Without<TabIndicator>),
    >,
    mut indicators: Query<(&TabIndicator, &mut Node, &mut BackgroundColor), Without<TabBarTab>>,
    mut labels: Query<(&ChildOf, &mut TextColor, &mut TextFont), With<TabBarTabLabel>>,
    states: Query<(&TabBarTab, &Hovered)>,
) {
    for (bar, mut node, mut background) in &mut bars {
        if bar_id.0.is_some_and(|id| id != bar) {
            continue;
        }
        let mut styled = node.clone();
        styled.border_radius = BorderRadius::all(theme.control_radius);
        // Write only on a difference: any write lays the tree out again.
        node.set_if_neq(styled);
        background.set_if_neq(BackgroundColor(theme.colors().segmented_track));
    }

    for (tab, mut node, mut background) in &mut tabs {
        if bar_id.0.is_some_and(|id| id != tab.owner()) {
            continue;
        }
        let mut styled = node.clone();
        styled.height = Val::Px(TAB_HEIGHT);
        styled.min_width = Val::Px(TAB_MIN_WIDTH);
        styled.padding = UiRect::axes(Val::Px(TAB_SIDE_PADDING), Val::ZERO);
        styled.align_items = AlignItems::Center;
        styled.justify_content = JustifyContent::Center;
        styled.border_radius = BorderRadius::all(theme.control_radius);
        node.set_if_neq(styled);
        background.set_if_neq(BackgroundColor(Color::NONE));
    }

    for (indicator, mut node, mut background) in &mut indicators {
        if bar_id.0.is_some_and(|id| id != indicator.owner) {
            continue;
        }
        let mut styled = node.clone();
        styled.border_radius = BorderRadius::all(theme.control_radius);
        node.set_if_neq(styled);
        background.set_if_neq(BackgroundColor(TabIndicator::color(&theme)));
    }

    for (child_of, mut text_color, mut text_font) in &mut labels {
        let Ok((tab, hovered)) = states.get(child_of.parent()) else {
            continue;
        };
        if bar_id.0.is_some_and(|id| id != tab.owner()) {
            continue;
        }
        let selected = selections
            .get(tab.owner())
            .is_some_and(|selection| selection.is_selected(tab.item()));
        text_color.set_if_neq(TextColor(
            segment_colors(theme.colors(), selected, hovered.get()).1,
        ));
        // Write only on a difference: any write lays the tree out again.
        text_font.set_if_neq(TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(TAB_FONT_SIZE),
            weight: FontWeight::SEMIBOLD,
            ..default()
        });
    }
}

/// Names each tab after its item's text.
fn sync_accessibility(
    mut tabs: ParamSet<(
        Query<
            (&TabBarTab, &mut AccessibilityNode),
            Or<(Changed<AccessibilityNode>, Changed<TabBarTab>)>,
        >,
        Query<(&TabBarTab, &mut AccessibilityNode)>,
    )>,
    items: Query<&Item<String>>,
    changed_items: Query<Entity, Changed<Item<String>>>,
) {
    for (tab, mut node) in &mut tabs.p0() {
        if let Ok(item) = items.get(tab.item()) {
            sync_label(&mut node, &item.data);
        }
    }
    let changed_items: EntityHashSet = changed_items.iter().collect();
    if changed_items.is_empty() {
        return;
    }
    for (tab, mut node) in &mut tabs.p1() {
        if changed_items.contains(&tab.item())
            && let Ok(item) = items.get(tab.item())
        {
            sync_label(&mut node, &item.data);
        }
    }
}

/// Sets the [`TabIndex`] of every tab to -1 while its bar is hidden, so Tab skips it.
///
/// The tab list's roving index is written only on a selection or focus change, so it
/// never hides or restores a bar.
fn sync_tab_index(
    hidden: Res<HiddenDirty>,
    bars: Query<(&SelectedTab, &Children), With<TabBar>>,
    mut tabs: ParamSet<(
        Query<
            &TabBarTab,
            Or<(
                Added<TabBarTab>,
                Changed<TabIndex>,
                Added<InteractionDisabled>,
            )>,
        >,
        Query<(&mut TabIndex, Has<InteractionDisabled>), With<TabBarTab>>,
    )>,
    focus: Res<InputFocus>,
    mut previous_focus: Local<Option<Entity>>,
    new_bars: Query<Entity, Added<TabBar>>,
    changed_bars: Query<Entity, (With<TabBar>, Or<(Changed<SelectedTab>, Changed<Children>)>)>,
    tab_owners: Query<&TabBarTab>,
    mut disabled_removed: RemovedComponents<InteractionDisabled>,
    shown: Query<(Option<&Node>, Option<&InheritedVisibility>)>,
    parents: Query<&ChildOf>,
) {
    let mut dirty = EntityHashSet::default();
    dirty.extend(new_bars.iter());
    dirty.extend(changed_bars.iter());
    dirty.extend(tabs.p0().iter().map(TabBarTab::owner));
    dirty.extend(
        disabled_removed
            .read()
            .filter_map(|tab| tab_owners.get(tab).ok().map(TabBarTab::owner)),
    );
    if focus.is_changed() || *previous_focus != focus.get() {
        dirty.extend(
            [*previous_focus, focus.get()]
                .into_iter()
                .flatten()
                .filter_map(|tab| tab_owners.get(tab).ok().map(TabBarTab::owner)),
        );
        *previous_focus = focus.get();
    }
    for bar in hidden.iter_with(&dirty) {
        let Ok((selected_tab, children)) = bars.get(bar) else {
            continue;
        };
        let mut tabs = tabs.p1();
        let enabled = |tab: &Entity| tabs.get(*tab).is_ok_and(|(_, disabled)| !disabled);
        let roving = if is_hidden(bar, &shown, &parents) {
            None
        } else {
            focus
                .get()
                .filter(|tab| children.contains(tab) && enabled(tab))
                .or_else(|| {
                    selected_tab
                        .0
                        .filter(|tab| children.contains(tab) && enabled(tab))
                })
                .or_else(|| children.iter().find(|tab| enabled(tab)))
        };
        for &tab in children {
            let Ok((mut index, _)) = tabs.get_mut(tab) else {
                continue;
            };
            let wanted = if roving == Some(tab) { 0 } else { -1 };
            if index.0 != wanted {
                index.0 = wanted;
            }
        }
    }
}

fn hover_change(tabs: Query<&TabBarTab, Changed<Hovered>>, mut commands: Commands) {
    for tab in &tabs {
        commands.run_system_cached_with(theme_tab_bars, Some(tab.owner()));
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
                .is_ok_and(|tab| selection.is_selected(tab.item()))
        });
    if selected_tab.0 != wanted {
        commands.entity(*bar).insert(SelectedTab(wanted));
    }
}

/// Keeps [`SelectedTab`] and the colors of the tabs in step with the selection.
fn selection_change(
    changed: Query<Entity, Changed<Selection>>,
    bars: Query<Entity, With<TabBar>>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let changed: EntityHashSet = changed.iter().collect();
    if changed.is_empty() {
        return;
    }
    for bar in &bars {
        if !changed.contains(&bar)
            && !selections
                .get_ref(bar)
                .is_some_and(|r| changed.contains(&r))
        {
            continue;
        }
        commands.run_system_cached_with(sync_selected_tab, bar);
        commands.run_system_cached_with(theme_tab_bars, Some(bar));
    }
}

struct ItemsChangedInput {
    bar_ref: Entity,
    items: Arc<[Entity]>,
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

    let wanted: HashSet<Entity> = items.iter().copied().collect();
    let mut shown = HashSet::new();
    for (tab_id, tab) in children
        .into_iter()
        .flat_map(RelationshipTarget::iter)
        .filter_map(|c| tabs.get(c).ok())
    {
        if !wanted.contains(&tab.item()) || !shown.insert(tab.item()) {
            commands.entity(tab_id).despawn();
        }
    }

    for &item_ref in items.iter().filter(|item_ref| !shown.contains(*item_ref)) {
        let Ok(item) = items_query.get(item_ref) else {
            continue;
        };
        commands.entity(*bar_ref).with_children(|spawner| {
            spawner
                .spawn((TabBarTab::new(item_ref, *bar_ref), Node::default()))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(item.data.clone()),
                        TabBarTabLabel,
                        Pickable::IGNORE,
                    ));
                });
        });
    }
    commands.run_system_cached_with(
        order_tabs,
        ItemsChangedInput {
            bar_ref: *bar_ref,
            items: Arc::clone(items),
        },
    );
    commands.run_system_cached_with(sync_selected_tab, *bar_ref);
    commands.run_system_cached_with(theme_tab_bars, Some(*bar_ref));
}

/// Puts the tabs of a bar in the order of its items.
fn order_tabs(
    input: In<ItemsChangedInput>,
    bars: Query<&Children, With<TabBar>>,
    tabs: Query<&TabBarTab>,
    mut commands: Commands,
) {
    let ItemsChangedInput { bar_ref, items } = &*input;
    let Ok(children) = bars.get(*bar_ref) else {
        return;
    };
    let wanted = items_in_order(children.iter(), items, |child| {
        tabs.get(child).ok().map(TabBarTab::item)
    });
    if !wanted.iter().copied().eq(children.iter()) {
        commands.entity(*bar_ref).replace_children(&wanted);
    }
}

/// Shows the current text of each tab's item.
fn sync_label_text(
    changed_items: Query<(), Changed<Item<String>>>,
    tabs: Query<&TabBarTab>,
    items: Query<&Item<String>>,
    mut labels: Query<(&ChildOf, &mut Text), With<TabBarTabLabel>>,
) {
    if changed_items.is_empty() {
        return;
    }
    for (child_of, mut text) in &mut labels {
        let Ok(tab) = tabs.get(child_of.parent()) else {
            continue;
        };
        if let Ok(item) = items.get(tab.item())
            && text.0 != item.data
        {
            text.0.clone_from(&item.data);
        }
    }
}

/// Clears the tabs of the bars whose items no longer resolve.
fn clear_dangling_tab_bars(
    mut removed_selections: RemovedComponents<Selection>,
    mut removed_refs: RemovedComponents<ItemsRef>,
    bars: Query<(Entity, &ItemsObserver), With<TabBar>>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let removed = removed_selections.read().count() + removed_refs.read().count() > 0;
    if removed {
        unwatch_dangling_items(&bars, &selections, &mut commands, |commands, bar_ref| {
            commands.run_system_cached_with(
                on_items_changed,
                ItemsChangedInput {
                    bar_ref,
                    items: Arc::from(Vec::<Entity>::new()),
                },
            );
        });
    }
}

fn on_insert_selection(
    t: On<Insert<Selection>>,
    bars: Query<(), With<TabBar>>,
    selections: SelectionParam,
    mut commands: Commands,
    watches: Query<&ItemsObserver>,
    observers: Query<(), With<Observer>>,
) {
    let bar_ref = t.event().event_target();
    if bars.get(bar_ref).is_err() {
        return;
    }
    let Some((values_ref, all_items)) = selections.get_items(bar_ref) else {
        return;
    };
    let on_changed: fn(&mut Commands, Entity, Arc<[Entity]>) = |commands, bar_ref, items| {
        commands.run_system_cached_with(on_items_changed, ItemsChangedInput { bar_ref, items });
    };
    if !watch_items(
        &mut commands,
        &watches,
        &observers,
        bar_ref,
        values_ref,
        on_changed,
    ) {
        return;
    }

    commands.run_system_cached_with(
        on_items_changed,
        ItemsChangedInput {
            bar_ref,
            items: all_items.iter().copied().collect(),
        },
    );
}
