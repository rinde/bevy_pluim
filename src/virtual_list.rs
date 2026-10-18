//! `VirtualList`: a list that renders only the items in view, for lists of many thousands.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::system::SystemParam;
use bevy::ecs::world::DeferredWorld;
use bevy::input::ButtonState;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusLost, FocusedInput, InputFocus, InputFocusVisible};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{Selected, UiSystems};
use bevy::ui_widgets::{ActiveDescendant, Button, ListItem};
use bevy::window::PrimaryWindow;
use itertools::Either;

use crate::focus::{AccessibilitySync, DrawFocusRing, TabStop};
use crate::list::{
    CornerStyle, ItemLabel, ItemViewCreator, ItemViewSpawner, SEPARATED_ITEM_PADDING,
    SEPARATED_LIST_PADDING, SEPARATED_RING_WIDTH, SEPARATED_ROW_GAP, item_background, item_border,
    sync_selected,
};
use crate::select::{Item, SelectionParam};
use crate::theme::Theme;
use crate::{MouseEvent, MouseKind};

/// The logical pixels of a [`Val::Px`], else 0.
const fn px(val: Val) -> f32 {
    match val {
        Val::Px(value) => value,
        _ => 0.0,
    }
}

/// Rows kept beyond each edge of the view, so a small scroll shows rows that are ready.
const OVERSCAN_ROWS: usize = 2;

/// How far a focus ring reaches beyond its row, in logical pixels: its 1 px offset and 2 px
/// width. The keyboard scrolls the active row into view with its ring.
const FOCUS_RING_REACH: f32 = 3.0;

pub(crate) struct VirtualListPlugin;

impl Plugin for VirtualListPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PostUpdate,
            (activate_on_focus, grow_pools, show_rows)
                .chain()
                .before(UiSystems::Layout)
                .before(DrawFocusRing),
        )
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
        .add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_virtual_lists)
                .run_if(resource_changed::<Theme>),
        )
        .add_observer(navigate)
        .add_observer(pick_on_click)
        .add_observer(deactivate_on_focus_lost)
        .register_type::<VirtualList>()
        .register_type::<VirtualListRow>();
    }
}

/// A list of rows of one height that renders only the items in view, so that it costs the
/// rows on screen, not the items.
///
/// It shows the items of its [`Selection`](crate::select::Selection) (its own or a
/// [`Selection::Ref`](crate::select::Selection::Ref)) as a list does, but its node is as tall
/// as all the rows together and holds only a pool of [`VirtualListRow`]s: those in view and
/// a few beyond. When the view moves, the rows are handed on to the items now in view; a row
/// is never spawned or despawned for a scroll. The pool grows when the view grows and stays at
/// that size, whatever the number of items.
///
/// Put it in a [`ScrollArea`](crate::scroll_area::ScrollArea) that scrolls vertically: the
/// view is the nearest ancestor whose [`Node::overflow`] scrolls along y, or the window when
/// there is none. It is as wide as its parent.
///
/// A row shows the text of an `Item<String>`. With an [`ItemViewCreator`] (added with the
/// list, before the first update) the app fills the rows instead: the creator runs with an
/// [`ItemViewSpawner`] whose `view` is the row, emptied, each time the row is handed to
/// another item. An app may also observe the insertion of [`VirtualListRow`], which happens
/// at the same moment.
///
/// Selection is that of a [`List`](crate::list::List): a row of a selected item carries
/// [`Selected`], a click on a row toggles its item as far as the selection allows, and once
/// the list has the focus (it is one tab stop) the arrow keys move the active row (wrapping
/// round), Page Up and Page Down by as many rows as fit in the view (stopping at the first
/// and last item), Home and End go to the first and last item, all scrolling the active row
/// into view, and Enter or Space selects its item. The focus ring is on the active row, which
/// [`ActiveDescendant`] names while it is in the pool, from the frame it moves.
///
/// With `separated` set it has the look of a [`List`](crate::list::List) of
/// [`CornerStyle::Separated`]: the rows stand apart, 6 px in from the list's edges with 2 px
/// between them, each rounded on all corners at the control radius with 10 by 12 px of padding,
/// and the selected row has a ring in the selection colour and the strong hover background. Rows
/// and ring are styled by the functions the list uses, so they cannot drift apart. The rows stay
/// absolutely placed: row `i` is at `6 + i * (row_height + 2)`, and the node is as tall as the
/// rows, gaps and padding together. `row_height` is then the height of a row itself, padding
/// and ring included, so it must exceed the 20 px of vertical padding and 2 of ring.
///
/// To a screen reader it is a [`Role::ListBox`] whose `size_of_set` is the number of items,
/// and each row in view a list item whose `position_in_set` is its item's place among them,
/// counted from 1, so a row says where it is among all items, not among the pooled rows.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Name::new("VirtualList"),
    Node = Node {
        width: Val::Percent(100.0),
        ..default()
    },
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::ListBox)),
    ActiveDescendant,
    TabStop,
    VirtualListState,
    Pickable {
        should_block_lower: false,
        ..default()
    },
)]
pub struct VirtualList {
    /// The height of every row, in logical pixels. A list whose row height is not a positive,
    /// finite number, or for a separated list not more than 22 px (the 20 px of vertical padding
    /// and the 2 px of ring), shows nothing and logs an error when it is inserted.
    pub row_height: f32,
    /// Whether the rows stand apart as in a [`CornerStyle::Separated`] list: padded, with gaps
    /// between them, rounded, and the selected one ringed.
    pub separated: bool,
}

impl VirtualList {
    /// A list whose rows are `row_height` logical pixels high.
    #[must_use]
    pub const fn new(row_height: f32) -> Self {
        Self {
            row_height,
            separated: false,
        }
    }

    /// A list of [`CornerStyle::Separated`] look, whose rows are `row_height` logical pixels
    /// high.
    #[must_use]
    pub const fn separated(row_height: f32) -> Self {
        Self {
            row_height,
            separated: true,
        }
    }

    /// The corner style whose row look this list has.
    const fn style(self) -> CornerStyle {
        if self.separated {
            CornerStyle::Separated
        } else {
            CornerStyle::FollowLayout
        }
    }

    /// The space between the list's edge and its first and last row, in logical pixels.
    const fn padding(self) -> f32 {
        if self.separated {
            px(SEPARATED_LIST_PADDING)
        } else {
            0.0
        }
    }

    /// The distance between the tops of neighbouring rows: a row and the gap after it.
    const fn pitch(self) -> f32 {
        if self.separated {
            self.row_height + px(SEPARATED_ROW_GAP)
        } else {
            self.row_height
        }
    }

    /// The top of row `index` in the list, in logical pixels.
    const fn row_top(self, index: usize) -> f32 {
        (index as f32).mul_add(self.pitch(), self.padding())
    }

    /// The height of the list with `count` rows: all rows, the gaps between them and the
    /// padding around.
    fn content_height(self, count: usize) -> f32 {
        if count == 0 {
            return 0.0;
        }
        let gaps = if self.separated {
            px(SEPARATED_ROW_GAP)
        } else {
            0.0
        };
        (count as f32).mul_add(self.pitch(), 2.0f32.mul_add(self.padding(), -gaps))
    }

    /// Sets the look of one row's node: where it lies across the list, padding, ring width and
    /// corners.
    fn style_row(self, node: &mut Node, theme: &Theme) {
        let inset = Val::Px(self.padding());
        node.left = inset;
        node.right = inset;
        if self.separated {
            node.padding = SEPARATED_ITEM_PADDING;
            node.border = UiRect::all(SEPARATED_RING_WIDTH);
            node.border_radius = BorderRadius::all(theme.control_radius);
        } else {
            node.padding = theme.item_padding;
            node.border = UiRect::ZERO;
            node.border_radius = BorderRadius::ZERO;
        }
    }

    /// Whether the row height is one rows can be laid out with: positive and finite, and for a
    /// separated list more than the vertical padding and ring of a row take up.
    const fn has_valid_row_height(self) -> bool {
        let floor = if self.separated {
            SEPARATED_ROW_CHROME
        } else {
            0.0
        };
        self.row_height > floor && self.row_height.is_finite()
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        if let Some(list) = world.get::<Self>(context.entity)
            && !list.has_valid_row_height()
        {
            error!(
                "VirtualList {} has row height {}, which is not a positive, finite number, or for a separated list not more than {} px of padding and ring; it shows nothing",
                context.entity, list.row_height, SEPARATED_ROW_CHROME
            );
        }
        world
            .commands()
            .run_system_cached_with(theme_virtual_lists, Some(context.entity));
    }
}

/// The height of a separated row's vertical padding (top and bottom of `SEPARATED_ITEM_PADDING`)
/// and ring (twice `SEPARATED_RING_WIDTH`): its height must exceed it.
const SEPARATED_ROW_CHROME: f32 = px(SEPARATED_ITEM_PADDING.top)
    + px(SEPARATED_ITEM_PADDING.bottom)
    + 2.0 * px(SEPARATED_RING_WIDTH);

/// One row of a [`VirtualList`], showing the item at `index` of the list's items.
///
/// Immutable: when the row is handed to another item a new value is inserted, so an app can
/// observe the change. A row that shows no item (the list has fewer items than rows) has
/// `None` for both and is not displayed.
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(immutable)]
#[require(
    Name::new("VirtualListRow"),
    Hovered,
    ListItem,
    Pickable {
        should_block_lower: false,
        ..default()
    },
)]
pub struct VirtualListRow {
    /// The list this row belongs to.
    pub list: Entity,
    /// Where the item it shows is in the list's items.
    pub index: Option<usize>,
    /// The item it shows.
    pub item: Option<Entity>,
}

/// The rows of a [`VirtualList`] and the index of the item the keyboard is on.
#[derive(Component, Debug, Clone, Default)]
struct VirtualListState {
    rows: Vec<Entity>,
    active: Option<usize>,
}

/// The text child of a row that shows an `Item<String>`.
#[derive(Component, Debug, Copy, Clone)]
struct RowLabel(Entity);

/// What of a [`VirtualList`] is in view, in logical pixels.
#[derive(Debug, Copy, Clone)]
struct View {
    /// The node that scrolls the list, if any.
    area: Option<Entity>,
    /// Where the list's top is in the scrolled contents of `area`.
    offset: f32,
    /// How far the contents of `area` are scrolled now.
    scroll: f32,
    /// How far they can scroll.
    max_scroll: f32,
    /// The height of the view.
    height: f32,
}

impl View {
    /// The first and last-but-one index of the rows to show, `rows` of them at most, for
    /// `count` items of `list`.
    #[expect(clippy::cast_sign_loss, reason = "the value is at least 0")]
    fn rows_in_view(&self, list: VirtualList, count: usize, rows: usize) -> std::ops::Range<usize> {
        let top = ((self.scroll - self.offset - list.padding()) / list.pitch())
            .floor()
            .max(0.0) as usize;
        let first = top
            .saturating_sub(OVERSCAN_ROWS)
            .min(count.saturating_sub(rows));
        first..first.saturating_add(rows).min(count)
    }

    /// How many whole rows of `list`, with their gaps, fit in the view, one at least: how far
    /// Page Up and Page Down move.
    #[expect(clippy::cast_sign_loss, reason = "the value is at least 1")]
    fn rows_per_page(&self, list: VirtualList) -> usize {
        (self.height / list.pitch()).floor().max(1.0) as usize
    }

    /// How many rows it takes to fill the view and the margin beyond it, for `count` items.
    #[expect(clippy::cast_sign_loss, reason = "the value is at least 0")]
    fn rows_needed(&self, list: VirtualList, count: usize) -> usize {
        let fitting = (self.height / list.pitch()).ceil().max(0.0) as usize;
        fitting.saturating_add(1 + 2 * OVERSCAN_ROWS).min(count)
    }
}

/// Finds what of a [`VirtualList`] is in view.
#[derive(SystemParam)]
struct Views<'w, 's> {
    parents: Query<'w, 's, &'static ChildOf>,
    lists: Query<'w, 's, (&'static ComputedNode, &'static UiGlobalTransform), With<VirtualList>>,
    areas: Query<
        'w,
        's,
        (
            &'static Node,
            &'static ComputedNode,
            &'static UiGlobalTransform,
            &'static ScrollPosition,
        ),
        (Without<VirtualList>, Without<VirtualListRow>),
    >,
    windows: Query<'w, 's, &'static Window, With<PrimaryWindow>>,
}

impl Views<'_, '_> {
    /// What of `list` is in view: the part of the nearest ancestor that scrolls along y, or of
    /// the window.
    ///
    /// The list's place comes from the last layout, the scroll from [`ScrollPosition`] as it
    /// is now, so a scroll shows its rows in the layout that follows it.
    fn of(&self, list: Entity) -> Option<View> {
        let (computed, transform) = self.lists.get(list).ok()?;
        let scale = computed.inverse_scale_factor();
        let list_top = computed.size().y.mul_add(-0.5, transform.translation.y) * scale;
        let area = self.parents.iter_ancestors(list).find(|ancestor| {
            self.areas
                .get(*ancestor)
                .is_ok_and(|(node, ..)| node.overflow.y == OverflowAxis::Scroll)
        });
        let Some((area, (_, area_computed, area_transform, scroll))) =
            area.and_then(|area| Some((area, self.areas.get(area).ok()?)))
        else {
            let height = self.windows.iter().next().map_or(0.0, Window::height);
            return Some(View {
                area: None,
                offset: list_top,
                scroll: 0.0,
                max_scroll: 0.0,
                height,
            });
        };
        let area_scale = area_computed.inverse_scale_factor();
        let area_height = area_computed.size().y * area_scale;
        let area_top = area_transform
            .translation
            .y
            .mul_add(area_scale, -0.5 * area_height);
        // The layout placed the list shifted by the scroll it had then.
        let laid_out_scroll = area_computed.scroll_position.y * area_scale;
        Some(View {
            area: Some(area),
            offset: list_top - area_top + laid_out_scroll,
            scroll: scroll.y,
            max_scroll: ((area_computed.content_size().y - area_computed.size().y) * area_scale)
                .max(0.0),
            height: area_height,
        })
    }
}

/// Spawns the rows a [`VirtualList`] lacks to fill its view, never more than it has items.
fn grow_pools(
    mut lists: Query<(
        Entity,
        &VirtualList,
        &mut VirtualListState,
        Option<&ItemViewCreator>,
    )>,
    views: Views,
    selections: SelectionParam,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for (list, virtual_list, mut state, creator) in &mut lists {
        if !virtual_list.has_valid_row_height() {
            continue;
        }
        let Some(count) = selections
            .get_item_values(list)
            .map(|(_, items)| items.len())
        else {
            continue;
        };
        let Some(view) = views.of(list) else {
            continue;
        };
        let needed = view.rows_needed(*virtual_list, count);
        while state.rows.len() < needed {
            let mut row = commands.spawn((
                VirtualListRow {
                    list,
                    index: None,
                    item: None,
                },
                {
                    let mut node = Node {
                        display: Display::None,
                        position_type: PositionType::Absolute,
                        height: Val::Px(virtual_list.row_height),
                        align_items: AlignItems::Center,
                        overflow: Overflow::clip(),
                        ..default()
                    };
                    virtual_list.style_row(&mut node, &theme);
                    node
                },
                BorderColor::all(Color::NONE),
                ChildOf(list),
            ));
            if creator.is_none() {
                let label = row
                    .commands()
                    .spawn((Text::default(), ItemLabel, Pickable::IGNORE))
                    .id();
                row.add_child(label).insert(RowLabel(label));
            }
            state.rows.push(row.id());
        }
    }
}

/// Hands the rows of each [`VirtualList`] to the items in view and keeps their place,
/// [`Selected`], colours and the [`ActiveDescendant`] in step.
///
/// Each frame it looks at the rows only, never at all the items: a row keeps its item while
/// that stays in view (row `i % rows` shows item `i`), so a scroll by one row hands on one
/// row.
fn show_rows(
    mut lists: Query<(
        Entity,
        &VirtualList,
        &VirtualListState,
        &ActiveDescendant,
        &mut Node,
        Option<&ItemViewCreator>,
    )>,
    mut rows: Query<
        (
            &VirtualListRow,
            &mut Node,
            &mut BackgroundColor,
            &mut BorderColor,
            &Hovered,
            Has<Selected>,
            Option<&RowLabel>,
        ),
        Without<VirtualList>,
    >,
    mut labels: Query<&mut Text>,
    texts: Query<Ref<Item<String>>>,
    views: Views,
    selections: SelectionParam,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for (list, virtual_list, state, active, mut list_node, creator) in &mut lists {
        if !virtual_list.has_valid_row_height() {
            continue;
        }
        let Some((_, items)) = selections.get_item_values(list) else {
            continue;
        };
        let Some(view) = views.of(list) else {
            continue;
        };
        let row_height = virtual_list.row_height;
        let height = Val::Px(virtual_list.content_height(items.len()));
        if list_node.height != height {
            list_node.height = height;
        }
        let selection = selections.get(list);

        let pool = state.rows.len();
        let range = view.rows_in_view(
            *virtual_list,
            items.len(),
            view.rows_needed(*virtual_list, items.len()).min(pool),
        );

        let mut active_row = None;
        for (slot, &row) in state.rows.iter().enumerate() {
            // The range is never longer than the pool, so at most one index in it has this
            // slot as its remainder: item `i` is shown by row `i % pool`.
            let index = Some(range.start + (slot + pool - range.start % pool) % pool)
                .filter(|index| *index < range.end);
            let Ok((current, mut node, mut background, mut border, hovered, was_selected, label)) =
                rows.get_mut(row)
            else {
                continue;
            };
            let item = index.and_then(|index| items.get(index).copied());
            let wanted = VirtualListRow { list, index, item };
            let handed_on = *current != wanted;
            if handed_on {
                commands.entity(row).insert(wanted);
                if let (Some(creator), Some(item)) = (creator, item) {
                    commands.entity(row).despawn_related::<Children>();
                    commands.run_system_with(creator.create, ItemViewSpawner { item, view: row });
                }
            }

            let display = if item.is_some() {
                Display::Flex
            } else {
                Display::None
            };
            let top = Val::Px(virtual_list.row_top(index.unwrap_or_default()));
            let row_height = Val::Px(row_height);
            if node.display != display || node.top != top || node.height != row_height {
                node.display = display;
                node.top = top;
                node.height = row_height;
            }

            if let (Some(RowLabel(label)), Some(text)) =
                (label, item.and_then(|item| texts.get(item).ok()))
                && (handed_on || text.is_changed())
                && let Ok(mut label_text) = labels.get_mut(*label)
            {
                label_text.0.clone_from(&text.data);
            }

            let is_selected = item
                .is_some_and(|item| selection.is_some_and(|selection| selection.is_selected(item)));
            sync_selected(&mut commands, row, was_selected, is_selected);
            background.set_if_neq(BackgroundColor(item_background(
                &theme.colors,
                virtual_list.style(),
                is_selected,
                hovered.get(),
            )));
            border.set_if_neq(BorderColor::all(item_border(
                &theme.colors,
                virtual_list.style(),
                is_selected,
            )));
            if index.is_some() && index == state.active {
                active_row = Some(row);
            }
        }
        if active.0 != active_row {
            commands.entity(list).insert(ActiveDescendant(active_row));
        }
    }
}

/// Picks the item of a clicked row, as far as the selection allows, and makes it the active
/// row. A click inside a [`Button`] in a row is the button's.
fn pick_on_click(
    t: On<MouseEvent>,
    parents: Query<&ChildOf>,
    rows: Query<&VirtualListRow>,
    buttons: Query<(), With<Button>>,
    mut states: Query<&mut VirtualListState>,
    mut selections: SelectionParam,
) {
    if !matches!(t.event().kind, MouseKind::Click(_)) {
        return;
    }
    let mut current = t.event().event_target();
    let row = loop {
        if let Ok(row) = rows.get(current) {
            break *row;
        }
        if buttons.contains(current) {
            return;
        }
        let Ok(child_of) = parents.get(current) else {
            return;
        };
        current = child_of.parent();
    };
    let (Some(index), Some(item)) = (row.index, row.item) else {
        return;
    };
    if let Ok(mut state) = states.get_mut(row.list) {
        state.active = Some(index);
    }
    selections.update_selection(row.list, |selection| selection.toggle_selected(item));
}

/// Moves the active row with the arrow keys, Page Up, Page Down, Home and End, scrolling it
/// into view, and selects its item on Enter or Space, while the list has the focus.
fn navigate(
    mut input: On<FocusedInput<KeyboardInput>>,
    focus: Res<InputFocus>,
    mut focus_visible: ResMut<InputFocusVisible>,
    mut lists: Query<(&VirtualList, &mut VirtualListState)>,
    views: Views,
    mut selections: SelectionParam,
    mut commands: Commands,
) {
    let list = input.event_target();
    if focus.get() != Some(list) {
        return;
    }
    let Ok((virtual_list, mut state)) = lists.get_mut(list) else {
        return;
    };
    if !virtual_list.has_valid_row_height() {
        return;
    }
    let key = &input.event().input;
    let is_move = matches!(
        key.key_code,
        KeyCode::ArrowUp
            | KeyCode::ArrowDown
            | KeyCode::ArrowLeft
            | KeyCode::ArrowRight
            | KeyCode::PageUp
            | KeyCode::PageDown
            | KeyCode::Home
            | KeyCode::End
    );
    let is_pick = matches!(key.key_code, KeyCode::Enter | KeyCode::Space);
    if !is_move && !is_pick {
        return;
    }
    // The list owns these keys while it has the focus: none goes on to the app's navigation.
    input.propagate(false);
    let key = input.event().input.clone();
    // A held arrow or page key keeps moving; Home, End, Enter and Space act on the first press.
    let repeats = !matches!(
        key.key_code,
        KeyCode::Home | KeyCode::End | KeyCode::Enter | KeyCode::Space
    );
    if key.state != ButtonState::Pressed || (key.repeat && !repeats) {
        return;
    }
    let Some((_, items)) = selections.get_item_values(list) else {
        return;
    };
    let count = items.len();
    let Some(last_index) = count.checked_sub(1) else {
        return;
    };
    let current = state.active.map(|index| index.min(last_index)).or_else(|| {
        let selection = selections.get(list)?;
        items.iter().position(|item| selection.is_selected(item))
    });

    if is_pick {
        let Some(item) = current.and_then(|index| items.get(index).copied()) else {
            return;
        };
        if selections
            .get(list)
            .is_some_and(|selection| !selection.is_selected(item))
        {
            selections.update_selection(list, |selection| selection.toggle_selected(item));
        }
        return;
    }

    let view = views.of(list);
    let page = view.map_or(1, |view| view.rows_per_page(*virtual_list));
    let next = match key.key_code {
        KeyCode::ArrowUp | KeyCode::ArrowLeft => match current {
            Some(index) if index > 0 && index <= last_index => index - 1,
            _ => last_index,
        },
        KeyCode::ArrowDown | KeyCode::ArrowRight => match current {
            Some(index) if index < last_index => index + 1,
            _ => 0,
        },
        KeyCode::PageUp => current.unwrap_or(0).saturating_sub(page),
        // With no active row Page Down starts where Arrow Down does, on the first item.
        KeyCode::PageDown => current.map_or(0, |index| index.saturating_add(page).min(last_index)),
        KeyCode::Home => 0,
        _ => last_index,
    };
    state.active = Some(next);
    focus_visible.0 = true;

    let Some(view) = view else {
        return;
    };
    let Some(area) = view.area else {
        return;
    };
    let row_top = view.offset + virtual_list.row_top(next) - FOCUS_RING_REACH;
    let row_bottom = 2.0f32.mul_add(FOCUS_RING_REACH, row_top + virtual_list.row_height);
    let scroll = if row_top < view.scroll {
        row_top
    } else if row_bottom > view.scroll + view.height {
        row_bottom - view.height
    } else {
        return;
    };
    // The list may have grown since the last layout measured how far the area scrolls.
    let max_scroll = view
        .max_scroll
        .max(view.offset + virtual_list.content_height(count) + FOCUS_RING_REACH - view.height);
    let mut position = ScrollPosition(Vec2::new(0.0, scroll.clamp(0.0, max_scroll.max(0.0))));
    if let Ok((_, _, _, current_position)) = views.areas.get(area) {
        position.x = current_position.x;
    }
    commands.entity(area).insert(position);
}

/// Makes the first selected item the active row when the list gets the focus, or the first
/// item when none is selected, as a list does.
///
/// A system that reads [`InputFocus`] rather than an observer of `FocusGained`: Bevy sends
/// that after the layout, so the row would be named and ringed a frame late, while the focus
/// itself (Tab, a click) is set in `PreUpdate`, before this runs ahead of the rows.
/// `InputFocus::is_changed` is true after any write to the focus, not only when it arrives
/// at the list, so a list focused while it had no items gets its first item made active the
/// next time anything writes the focus.
fn activate_on_focus(
    focus: Res<InputFocus>,
    mut lists: Query<&mut VirtualListState>,
    selections: SelectionParam,
) {
    if !focus.is_changed() {
        return;
    }
    let Some(list) = focus.get() else {
        return;
    };
    let Ok(mut state) = lists.get_mut(list) else {
        return;
    };
    if state.active.is_some() {
        return;
    }
    let Some((_, items)) = selections.get_item_values(list) else {
        return;
    };
    if items.is_empty() {
        return;
    }
    let first_selected = selections
        .get(list)
        .and_then(|selection| items.iter().position(|item| selection.is_selected(item)));
    state.active = Some(first_selected.unwrap_or(0));
}

/// Clears the active row when the list loses the focus, and the ring goes with it.
fn deactivate_on_focus_lost(
    focus_lost: On<FocusLost>,
    focus: Res<InputFocus>,
    mut lists: Query<&mut VirtualListState>,
) {
    let list = focus_lost.event_target();
    if focus.get() == Some(list) {
        return;
    }
    if let Ok(mut state) = lists.get_mut(list) {
        state.active = None;
    }
}

/// Tells a screen reader how many items each [`VirtualList`] has and where the item of each
/// of its rows is among them, so a row in the pool reads as item 9,000 of 10,000.
fn sync_accessibility(
    mut lists: Query<(Entity, &VirtualListState, &mut AccessibilityNode), With<VirtualList>>,
    mut rows: Query<(&VirtualListRow, &mut AccessibilityNode), Without<VirtualList>>,
    selections: SelectionParam,
) {
    for (list, state, mut node) in &mut lists {
        let count = selections
            .get_item_values(list)
            .map_or(0, |(_, items)| items.len());
        if node.size_of_set() != Some(count) {
            node.set_size_of_set(count);
        }
        for &row in &state.rows {
            let Ok((row, mut node)) = rows.get_mut(row) else {
                continue;
            };
            let position = row.index.map(|index| index + 1);
            match position {
                Some(position) if node.position_in_set() != Some(position) => {
                    node.set_position_in_set(position);
                }
                None if node.position_in_set().is_some() => node.clear_position_in_set(),
                _ => {}
            }
            let size = position.map(|_| count);
            match size {
                Some(size) if node.size_of_set() != Some(size) => node.set_size_of_set(size),
                None if node.size_of_set().is_some() => node.clear_size_of_set(),
                _ => {}
            }
        }
    }
}

fn theme_virtual_lists(
    list: In<Option<Entity>>,
    theme: Res<Theme>,
    mut lists: Query<(&mut BackgroundColor, &VirtualList, &VirtualListState)>,
    mut rows: Query<&mut Node, With<VirtualListRow>>,
) {
    let it = match list.0 {
        Some(list) => Either::Left(lists.get_mut(list).into_iter()),
        None => Either::Right(lists.iter_mut()),
    };
    for (mut background, virtual_list, state) in it {
        background.0 = theme.colors.background_primary;
        for &row in &state.rows {
            if let Ok(mut node) = rows.get_mut(row) {
                virtual_list.style_row(&mut node, &theme);
            }
        }
    }
}
