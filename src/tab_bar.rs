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
use crate::select::{Item, ItemsObserver, Selection, SelectionParam, items_in_order, watch_items};
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
                // After the tab list's own roving `TabIndex`, which it writes through commands
                // before `FocusChangeEvents`, so the hidden bar's -1 is the last word.
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

/// The highlight behind the selected tab of a [`TabBar`]: a child of the bar, placed absolutely
/// behind the tabs, which slides to the selected tab when the selection changes.
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
/// Only the widget builds one, and the component is immutable: the item and the bar are fixed
/// for the life of the tab and are read with [`item`](Self::item) and [`owner`](Self::owner).
/// A tab's accessible name is its item's text, so callers supply meaningful item labels.
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

/// Marks the text of a [`TabBarTab`].
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
struct TabBarTabLabel;

/// Keeps the keys a focused tab moves the focus with (left, right, Home, End) from
/// bubbling on to an app's directional navigation: the tab list stops a first press only,
/// so this stops the repeats and releases as well. Up and down go on, as a row of tabs
/// does not use them.
fn keep_navigation_keys(
    mut input: On<FocusedInput<KeyboardInput>>,
    tabs: Query<(), With<TabBarTab>>,
    focus: Res<InputFocus>,
) {
    // `focused_entity` follows the event up the tree, so the focus itself is read.
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
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = theme.colors().segmented_track;
    }

    for (tab, mut node, mut background) in &mut tabs {
        if bar_id.0.is_some_and(|id| id != tab.owner()) {
            continue;
        }
        node.height = Val::Px(TAB_HEIGHT);
        node.min_width = Val::Px(TAB_MIN_WIDTH);
        node.padding = UiRect::axes(Val::Px(TAB_SIDE_PADDING), Val::ZERO);
        node.align_items = AlignItems::Center;
        node.justify_content = JustifyContent::Center;
        node.border_radius = BorderRadius::all(theme.control_radius);
        // The selected look is the indicator's, which slides.
        background.0 = Color::NONE;
    }

    for (indicator, mut node, mut background) in &mut indicators {
        if bar_id.0.is_some_and(|id| id != indicator.owner) {
            continue;
        }
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = TabIndicator::color(&theme);
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
        text_color.0 = segment_colors(theme.colors(), selected, hovered.get()).1;
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
    // A tab is looked at when Bevy reset its node or it is new, and when its item's text
    // changed; an idle frame touches none.
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

/// Keeps the [`TabIndex`] of the tabs of every bar: while the bar is shown, the tab list's
/// roving one (Bevy's `update_tablist_derived_state`: 0 for the focused tab, else the selected
/// one, else the first enabled one, -1 for the others), and -1 for every tab while the bar is hidden (its [`InheritedVisibility`]
/// is hidden, or it or an ancestor is `Display::None`, as in a hidden `Stack` panel), so Tab
/// skips a hidden bar and reaches it again once it shows.
///
/// The tab list writes its roving index only when the selection, the tabs or the focus change,
/// so it would neither hide a bar nor bring one back. This looks only at the bars that may
/// need a different answer: a new bar or tab, a changed `SelectedTab` or `Children`, a tab
/// whose `InteractionDisabled` or `TabIndex` changed, a move of the focus (the bars of the
/// tab that had it and the one that has it), and a bar whose visibility, `Display::None` (on
/// it or an ancestor) or parent changed. A quiet frame does no work.
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

/// Restyles the tabs of a bar when a tab gains or loses the pointer.
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

/// Keeps [`SelectedTab`] and the colours of the tabs in step with the selection, own or
/// referenced.
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
    // Only the bars whose own selection, or the one they refer to, changed.
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

    // Linear reconciliation (see `List`): a tab stays when its item is wanted and no
    // earlier tab shows it; any other is despawned.
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

/// Puts the tabs of a bar in the order of its items. Children are only reordered, and only when
/// they differ.
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

/// Shows the current text of each tab's item: a changed [`Item<String>`] updates its
/// [`TabBarTabLabel`] in place.
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
