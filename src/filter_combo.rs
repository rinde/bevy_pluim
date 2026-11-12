//! `FilterCombo`: a [`Combo`](crate::combo::Combo) whose popup has a search box that filters
//! the options as a person types.
//!
//! The popup is a card holding a search field above a scrolling [`List`] of the options, and a
//! "No matching items" line that shows when nothing matches. The keyboard focus stays in the
//! search field while the popup is open: typing edits the query, Arrow Down and Arrow Up move
//! the active option through the options that match, and Enter picks it. The query and the
//! active option are the widget's own: the [`Selection`] and its items are never changed by
//! filtering, and another widget showing the same items sees all of them.

use accesskit::{Live, Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusCause, FocusGained, FocusLost, FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextEdit};
use bevy::ui::{ComputedNode, UiGlobalTransform, UiSystems};
use bevy::ui_widgets::{Activate, ActiveDescendant};

use crate::MouseEvent;
use crate::combo::{
    CHEVRON_SIZE, ComboButton, ComboList, ComboPlaceholder, ComboState, PLACEHOLDER_TEXT, chevron,
    selected_text,
};
use crate::float::{Float, UiViewport, fit_vertically, keep_in_window_x};
use crate::focus::{
    AccessibilitySync, FOCUS_RING_OFFSET, FOCUS_RING_REACH, FOCUS_RING_WIDTH, Unreachable, node_id,
    sync_controls, sync_label, sync_role, sync_value,
};
use crate::icon::Icon;
use crate::list::{CornerStyle, ItemView, List};
use crate::popup::{self, ButtonLook};
use crate::push_button::{NoDefaultHover, PushButton, PushButtonLabel, PushButtonRefs};
use crate::scroll_area::{PopupScrollBar, popup_scroll_bar, popup_scroll_bar_hovered};
use crate::select::{Item, Selection, SelectionParam, SelectionValue};
use crate::text_field::{OwnKeys, TextField, TextFieldChange, TextFieldRefs};
use crate::theme::Theme;

/// Registers the systems and observers of [`FilterCombo`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct FilterComboPlugin;

impl Plugin for FilterComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                close_on_outside_click,
                selection_change,
                (mark_dirty, apply_filters).chain(),
                style_active_options,
                (|| None::<Entity>)
                    .pipe(theme_filter_combos)
                    .run_if(resource_changed::<Theme>),
            ),
        )
        .add_systems(
            PostUpdate,
            (
                sync_accessibility.in_set(AccessibilitySync),
                place_open_popups.before(UiSystems::Layout),
                keep_active_in_view.after(UiSystems::PostLayout),
            ),
        )
        .add_observer(on_state)
        .add_observer(on_field_change)
        .add_observer(on_insert_filter_combo);
    }
}

/// The text of the zero-results line.
const NO_MATCHES_TEXT: &str = "No matching items";

/// The placeholder of the search box.
const SEARCH_PLACEHOLDER: &str = "Search..";

/// The accessible name of the search box when the combo has no name.
const SEARCH_NAME: &str = "Search";

/// The padding of the popup around the search box and the list, in logical pixels. It is also
/// the padding of the list itself (`CornerStyle::Separated`), which the active option brings
/// into view with it.
const POPUP_PADDING: f32 = 6.0;

/// How many frames after a change the active option is kept in view: the layout that moves
/// it comes one frame later.
const SETTLE_FRAMES: u8 = 3;

/// A button that opens a popup with a search box and a [`List`] of the options it filters.
///
/// It is used like a [`Combo`](crate::combo::Combo), on a node with a [`Selection`] (its own,
/// or a [`Selection::reference`] to one it shares), and shows the selected items on its
/// button in the same way, with a [`ComboPlaceholder`] while nothing is selected. Opening the
/// popup starts with an empty query and puts the keyboard focus in the search box.
///
/// # Filtering
///
/// The query is text typed in the search box, not the selected value. The options whose
/// `Item<String>` text contains it, ignoring case, stay; the others are hidden (and left out
/// of the keyboard navigation and the accessibility tree), in the order of the source items. An
/// empty query shows all of them. A note ([`ItemNote`](crate::list::ItemNote)) shows but is not
/// searched. With no match the popup says "No matching items". The filter belongs to this
/// widget: the options are the original item entities, so another widget on the same items
/// shows all of them, and filtering never changes the [`Selection`].
///
/// # Keyboard
///
/// - Typing, Backspace, Delete, Left, Right, Home, End, selection and the clipboard shortcuts
///   edit the query; Space types a space.
/// - Arrow Down and Arrow Up move the active option through the matching options, wrapping,
///   and repeat when held. They change nothing else and never reach an app's directional
///   navigation.
/// - Enter (also the numpad's) picks the active option as a click on it would: a selection of
///   one item at most closes the popup and gives the focus back to the button; a selection of
///   many stays open with the query so that more can be picked. With no active option (nothing
///   matches) it does nothing.
/// - Escape closes the popup without picking. Tab and Shift+Tab, and a click elsewhere, close
///   it without picking too, and the focus goes where they send it. Reopening starts with an
///   empty query.
/// - While an input method is composing text, none of these keys picks or closes anything.
///
/// The active option starts as the first selected matching option, otherwise the first match.
/// After the query or the items change it stays when it still matches, otherwise it becomes
/// the first match, or none.
///
/// To assistive technology the button is a combo box whose value is the selected items, and the
/// search box is a combo box whose value is the query, which controls the list and has the
/// active option as its active descendant. `name`, if given, is the label of both.
#[derive(Component, Debug, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add, on_insert = Self::on_insert)]
#[require(
    Node {
        width: Val::Px(200.0),
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Name::new("FilterCombo"),
)]
pub struct FilterCombo {
    /// The accessible name of the combo, the label a screen reader reads. The search box is
    /// named "Search" without one.
    pub name: Option<String>,
}

impl FilterCombo {
    /// The same combo, with `name` as its accessible name.
    #[must_use]
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let root = context.entity;
        let combo_name = world.get::<Self>(root).and_then(|combo| combo.name.clone());
        let placeholder = world
            .get::<ComboPlaceholder>(root)
            .map_or_else(|| PLACEHOLDER_TEXT.to_string(), |p| p.0.clone());
        let chevron = world
            .get_resource_mut::<Assets<Image>>()
            .map_or_else(Handle::default, |mut images| images.add(chevron()));
        let mut commands = world.commands();

        let button = commands
            .spawn((
                PushButton::from_label(placeholder),
                ComboButton,
                NoDefaultHover,
                FilterPart(root),
                // Lets the pointer through to the combo, as a `Combo`'s button does.
                Pickable {
                    should_block_lower: false,
                    ..default()
                },
                ChildOf(root),
            ))
            .with_children(|button| {
                button.spawn((
                    Icon::bundle(chevron, CHEVRON_SIZE),
                    Name::new("Chevron"),
                    Pickable::IGNORE,
                ));
            })
            .id();
        let popup = commands
            .spawn((
                Name::new("FilterComboPopup"),
                Float,
                Node {
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(POPUP_PADDING)),
                    row_gap: Val::Px(POPUP_PADDING),
                    ..default()
                },
                BackgroundColor::default(),
                BorderColor::default(),
                // A popup floats over what is below it, so it takes the pointer from there.
                Pickable::default(),
                Hovered::default(),
                Visibility::Hidden,
                FilterPart(root),
                ChildOf(root),
            ))
            .id();
        let field = commands
            .spawn((
                search_field(combo_name.as_deref()),
                Hovered::default(),
                FilterPart(root),
                ChildOf(popup),
            ))
            .id();
        let list = commands
            .spawn((
                List,
                CornerStyle::Separated,
                ComboList,
                FilterList,
                FilterPart(root),
                // Never a tab stop: the search box has the focus while the popup is open.
                Unreachable,
                Node {
                    overflow: Overflow::scroll_y(),
                    min_height: Val::Px(0.0),
                    ..default()
                },
                ChildOf(popup),
            ))
            .id();
        // The list scrolls itself when the popup is capped to the window: it gets a bar.
        commands.spawn(popup_scroll_bar(list));
        let empty = commands
            .spawn((
                Name::new("FilterComboEmpty"),
                Text::new(NO_MATCHES_TEXT),
                FilterEmpty,
                FilterPart(root),
                Hovered::default(),
                Node {
                    display: Display::None,
                    padding: UiRect::axes(Val::Px(12.0), Val::Px(10.0)),
                    ..default()
                },
                empty_accessibility(),
                ChildOf(popup),
            ))
            .id();
        commands.run_system_cached_with(
            finish_setup,
            Parts {
                root,
                button,
                popup,
                field,
                list,
                empty,
            },
        );
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(apply_name, context.entity);
    }
}

/// The parts of a [`FilterCombo`], on the combo itself. Only the widget builds it.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct FilterComboRefs {
    button: Entity,
    popup: Entity,
    field: Entity,
    input: Entity,
    list: Entity,
    empty: Entity,
}

impl FilterComboRefs {
    /// The button that opens and closes the popup.
    #[must_use]
    pub const fn button(&self) -> Entity {
        self.button
    }

    /// The popup, a [`Float`] holding the search box, the list and the zero-results line.
    #[must_use]
    pub const fn popup(&self) -> Entity {
        self.popup
    }

    /// The [`TextField`] of the search box.
    #[must_use]
    pub const fn field(&self) -> Entity {
        self.field
    }

    /// The text input inside the search box, which has the keyboard focus while the popup is
    /// open.
    #[must_use]
    pub const fn input(&self) -> Entity {
        self.input
    }

    /// The [`List`] of all the options, those that do not match the query hidden.
    #[must_use]
    pub const fn list(&self) -> Entity {
        self.list
    }

    /// The "No matching items" line.
    #[must_use]
    pub const fn empty(&self) -> Entity {
        self.empty
    }
}

/// The part of a [`FilterCombo`] this entity is, pointing at the combo.
#[derive(Component, Debug, Copy, Clone)]
struct FilterPart(Entity);

/// Marks the [`List`] in the popup.
#[derive(Component, Debug, Copy, Clone)]
struct FilterList;

/// Marks the zero-results line.
#[derive(Component, Debug, Copy, Clone)]
struct FilterEmpty;

/// Marks the row of the active option, which has the focus ring drawn on it.
#[derive(Component, Debug, Copy, Clone)]
struct ActiveOption;

/// Marks a combo whose results must be worked out again.
#[derive(Component, Debug, Copy, Clone)]
struct FilterDirty;

/// The option the keyboard is on: the item and its row.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Active {
    item: Entity,
    view: Entity,
}

/// What a [`FilterCombo`] remembers while it is open.
#[derive(Component, Debug, Clone, Default)]
struct FilterState {
    /// The text in the search box.
    query: String,
    /// The active option, if any option matches.
    active: Option<Active>,
    /// Whether the popup has just opened, so that the active option starts at the selection.
    fresh: bool,
    /// How many more frames to keep the active option in view.
    settle: u8,
}

#[derive(Debug, Copy, Clone)]
struct Parts {
    root: Entity,
    button: Entity,
    popup: Entity,
    field: Entity,
    list: Entity,
    empty: Entity,
}

/// The search box for a combo with this name.
fn search_field(name: Option<&str>) -> TextField {
    TextField::new("")
        .with_placeholder(SEARCH_PLACEHOLDER)
        .with_name(name.unwrap_or(SEARCH_NAME))
        .with_min_width(Val::Px(0.0))
        .with_max_width(Val::Auto)
}

/// The accessibility node of the zero-results line: a polite status.
fn empty_accessibility() -> AccessibilityNode {
    let mut node = AccessNode::new(Role::Status);
    node.set_label(NO_MATCHES_TEXT);
    node.set_live(Live::Polite);
    AccessibilityNode::from(node)
}

/// Finishes the setup once the search field has built its input: records the parts and wires
/// the observers.
fn finish_setup(parts: In<Parts>, fields: Query<&TextFieldRefs>, mut commands: Commands) {
    let Parts {
        root,
        button,
        popup,
        field,
        list,
        empty,
    } = parts.0;
    let Ok(field_refs) = fields.get(field) else {
        error!("the search field of FilterCombo {root} has no input");
        return;
    };
    let input = field_refs.input;
    commands
        .entity(input)
        .insert((FilterPart(root), OwnKeys))
        .observe(on_key)
        .observe(on_focus_lost);
    commands.entity(list).observe(on_list_focus);
    commands.entity(button).observe(
        move |_: On<Activate>, states: Query<&ComboState>, mut commands: Commands| {
            if let Ok(state) = states.get(root) {
                commands.entity(root).insert(state.inverse());
            }
        },
    );
    commands
        .entity(root)
        .insert((
            FilterComboRefs {
                button,
                popup,
                field,
                input,
                list,
                empty,
            },
            FilterState::default(),
            ComboState::Closed,
        ))
        .observe(handle_button_mouse_events);
    commands.run_system_cached_with(theme_filter_combos, Some(root));
}

/// Puts the name of a combo on its button and its search box.
fn apply_name(
    root: In<Entity>,
    combos: Query<(&FilterCombo, &FilterComboRefs)>,
    mut commands: Commands,
) {
    let Ok((combo, refs)) = combos.get(root.0) else {
        return;
    };
    commands
        .entity(refs.field)
        .insert(search_field(combo.name.as_deref()));
}

fn on_insert_filter_combo(
    t: On<Insert<FilterCombo>>,
    refs: Query<(), With<FilterComboRefs>>,
    mut commands: Commands,
) {
    // A combo inserted again keeps its parts; its name is applied by the hook.
    let root = t.event().event_target();
    if refs.contains(root) {
        commands.entity(root).insert(FilterDirty);
    }
}

/// Shows or hides the popup as the state says, and starts or ends a search.
fn on_state(
    t: On<Insert<ComboState>>,
    mut combos: Query<(
        &FilterComboRefs,
        &ComboState,
        &mut FilterState,
        &FilterCombo,
    )>,
    mut focus: ResMut<InputFocus>,
    mut nodes: Query<&mut AccessibilityNode>,
    mut commands: Commands,
) {
    let root = t.event().event_target();
    let Ok((refs, &state, mut filter, combo)) = combos.get_mut(root) else {
        return;
    };
    let open = state == ComboState::Open;
    popup::show_popup(&mut commands, open, refs.input, refs.popup);
    if open {
        *filter = FilterState {
            fresh: true,
            ..default()
        };
        // Sets the text without a change event; the query starts empty.
        commands
            .entity(refs.field)
            .insert(search_field(combo.name.as_deref()));
        commands.entity(root).insert(FilterDirty);
        focus.set(refs.input, FocusCause::Navigated);
    } else {
        set_active(&mut commands, &mut filter, refs.input, None);
        // The focus never stays on something hidden; where a click or Tab sent it, it stays.
        let on_popup = focus
            .get()
            .is_some_and(|focused| focused == refs.input || focused == refs.list);
        if on_popup {
            focus.set(refs.button, FocusCause::Navigated);
        }
    }
    if let Ok(mut node) = nodes.get_mut(refs.button)
        && node.is_expanded() != Some(open)
    {
        popup::set_expanded(&mut node, open);
    }
}

/// Marks the open combos whose rows or items changed.
fn mark_dirty(
    lists: Query<&FilterPart, (With<FilterList>, Changed<Children>)>,
    renamed: Query<(), Changed<Item<String>>>,
    combos: Query<(Entity, &ComboState), With<FilterCombo>>,
    mut commands: Commands,
) {
    for part in &lists {
        if let Ok(mut combo) = commands.get_entity(part.0) {
            combo.insert(FilterDirty);
        }
    }
    if !renamed.is_empty() {
        for (combo, state) in &combos {
            if *state == ComboState::Open {
                commands.entity(combo).insert(FilterDirty);
            }
        }
    }
}

/// Whether the text of an option matches the query: it contains it, ignoring case.
fn matches(label: &str, lowercase_query: &str) -> bool {
    lowercase_query.is_empty() || label.to_lowercase().contains(lowercase_query)
}

/// Hides the rows that do not match the query, and keeps the active option on a match.
fn apply_filters(
    dirty: Query<Entity, With<FilterDirty>>,
    mut combos: Query<(&FilterComboRefs, &ComboState, &mut FilterState)>,
    children: Query<&Children>,
    mut rows: Query<(&ItemView, &mut Node), Without<FilterEmpty>>,
    items: Query<&Item<String>>,
    mut empties: Query<&mut Node, (With<FilterEmpty>, Without<ItemView>)>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    for root in &dirty {
        commands.entity(root).remove::<FilterDirty>();
        let Ok((refs, &state, mut filter)) = combos.get_mut(root) else {
            continue;
        };
        if state != ComboState::Open {
            continue;
        }
        let query = filter.query.to_lowercase();
        let mut shown = Vec::new();
        for row in children
            .get(refs.list)
            .into_iter()
            .flat_map(RelationshipTarget::iter)
        {
            let Ok((view, mut node)) = rows.get_mut(row) else {
                continue;
            };
            let found = items
                .get(view.item_ref)
                .is_ok_and(|item| matches(&item.data, &query));
            let display = if found { Display::Flex } else { Display::None };
            if node.display != display {
                node.display = display;
            }
            if found {
                shown.push(Active {
                    item: view.item_ref,
                    view: row,
                });
            }
        }

        let kept = filter.active.and_then(|active| {
            shown
                .iter()
                .find(|shown| shown.item == active.item)
                .copied()
        });
        let next = kept.or_else(|| {
            let selection = if filter.fresh {
                selections.get(refs.list)
            } else {
                None
            };
            selection
                .and_then(|selection| shown.iter().find(|shown| selection.is_selected(shown.item)))
                .or_else(|| shown.first())
                .copied()
        });
        if !shown.is_empty() {
            filter.fresh = false;
        }
        set_active(&mut commands, &mut filter, refs.input, next);

        if let Ok(mut node) = empties.get_mut(refs.empty) {
            let display = if shown.is_empty() {
                Display::Flex
            } else {
                Display::None
            };
            if node.display != display {
                node.display = display;
            }
        }
        filter.settle = SETTLE_FRAMES;
    }
}

/// Makes `next` the active option, moving the marker from the old row to the new one.
///
/// The search box also gets Bevy's [`ActiveDescendant`] naming that row, as a `ListBox` has one,
/// so a check can read the active option the same way for both.
fn set_active(
    commands: &mut Commands,
    filter: &mut FilterState,
    input: Entity,
    next: Option<Active>,
) {
    if filter.active == next {
        return;
    }
    if let Ok(mut input) = commands.get_entity(input) {
        input.insert(ActiveDescendant(next.map(|active| active.view)));
    }
    if let Some(old) = filter.active
        && let Ok(mut row) = commands.get_entity(old.view)
    {
        row.remove::<ActiveOption>();
    }
    if let Some(new) = next
        && let Ok(mut row) = commands.get_entity(new.view)
    {
        row.insert(ActiveOption);
    }
    filter.active = next;
}

/// Takes the new text of a search box as the query.
fn on_field_change(
    t: On<TextFieldChange>,
    parts: Query<&FilterPart>,
    mut states: Query<&mut FilterState>,
    mut commands: Commands,
) {
    let Ok(part) = parts.get(t.event().entity) else {
        return;
    };
    let Ok(mut filter) = states.get_mut(part.0) else {
        return;
    };
    if filter.query != t.event().value {
        filter.query.clone_from(&t.event().value);
        filter.fresh = false;
        commands.entity(part.0).insert(FilterDirty);
    }
}

/// The rows that show, in order, with their items.
fn shown_rows(
    list: Entity,
    children: &Query<&Children>,
    rows: &Query<(&ItemView, &Node), Without<FilterEmpty>>,
) -> Vec<Active> {
    children
        .get(list)
        .into_iter()
        .flat_map(RelationshipTarget::iter)
        .filter_map(|row| {
            let (view, node) = rows.get(row).ok()?;
            (node.display != Display::None).then_some(Active {
                item: view.item_ref,
                view: row,
            })
        })
        .collect()
}

/// The keys the search box handles: Arrow Down and Arrow Up move the active option, Enter picks
/// it. Everything else edits the query, or is left to the text input.
fn on_key(
    mut input: On<FocusedInput<KeyboardInput>>,
    parts: Query<&FilterPart>,
    mut combos: Query<(&FilterComboRefs, &ComboState, &mut FilterState)>,
    mut editables: Query<&mut EditableText>,
    children: Query<&Children>,
    rows: Query<(&ItemView, &Node), Without<FilterEmpty>>,
    mut selections: SelectionParam,
    mut commands: Commands,
) {
    let target = input.event_target();
    let (Ok(&FilterPart(root)), Ok(mut editable)) = (parts.get(target), editables.get_mut(target))
    else {
        return;
    };
    // While an input method composes, its keys are its own.
    if editable.is_composing() {
        return;
    }
    let Ok((refs, &state, mut filter)) = combos.get_mut(root) else {
        return;
    };
    if state != ComboState::Open {
        return;
    }
    let key = &input.event().input;
    let (pressed, repeat, key_code) = (key.state.is_pressed(), key.repeat, key.key_code);
    match key_code {
        KeyCode::ArrowDown | KeyCode::ArrowUp => {
            input.propagate(false);
            if !pressed {
                return;
            }
            // The text input moved the caret for this key as well; the key is ours.
            if matches!(
                editable.pending_edits.last(),
                Some(TextEdit::Up(_) | TextEdit::Down(_))
            ) {
                editable.pending_edits.pop();
            }
            let shown = shown_rows(refs.list, &children, &rows);
            if shown.is_empty() {
                return;
            }
            let at = filter
                .active
                .and_then(|active| shown.iter().position(|shown| shown.item == active.item));
            let last = shown.len() - 1;
            let next = match (key_code, at) {
                (KeyCode::ArrowDown, Some(at)) => (at + 1) % shown.len(),
                (KeyCode::ArrowDown, None) => 0,
                (_, Some(at)) => at.checked_sub(1).unwrap_or(last),
                (_, None) => last,
            };
            set_active(
                &mut commands,
                &mut filter,
                refs.input,
                shown.get(next).copied(),
            );
            filter.fresh = false;
            filter.settle = SETTLE_FRAMES;
        }
        KeyCode::Enter | KeyCode::NumpadEnter => {
            input.propagate(false);
            if !pressed || repeat {
                return;
            }
            let Some(active) = filter.active else {
                return;
            };
            let Some(multiple) = selections
                .get(refs.list)
                .map(SelectionValue::allow_multiple)
            else {
                return;
            };
            selections.update_selection(refs.list, |selection| {
                selection.toggle_selected(active.item)
            });
            if !multiple {
                commands.entity(root).insert(ComboState::Closed);
            }
        }
        // By key code, as `TextField` does: the text input reads the logical key, which not
        // every source of key events fills in.
        KeyCode::Escape => {
            input.propagate(false);
            if pressed {
                commands.entity(root).insert(ComboState::Closed);
            }
        }
        _ => {}
    }
}

/// Closes the popup when the search box loses the focus other than to the popup itself.
fn on_focus_lost(
    lost: On<FocusLost>,
    parts: Query<&FilterPart>,
    combos: Query<(&FilterComboRefs, &ComboState)>,
    hovered: Query<&Hovered>,
    mouse: Res<ButtonInput<MouseButton>>,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let Ok(&FilterPart(root)) = parts.get(lost.event().entity) else {
        return;
    };
    let Ok((refs, &state)) = combos.get(root) else {
        return;
    };
    let over = |entity: Entity| hovered.get(entity).is_ok_and(Hovered::get);
    let open = state == ComboState::Open;
    if open {
        // The focus going to the button: by a press on it, the button toggles the popup itself
        // on release; by Shift+Tab, the popup closes.
        if focus.get() == Some(refs.button) {
            if !(over(refs.button) && mouse.pressed(MouseButton::Left)) {
                commands.entity(root).insert(ComboState::Closed);
            }
            return;
        }
        // A click in the popup takes the focus off the search box (the list, the empty line
        // and the card are not where typing goes): it stays open and the box gets it back.
        if over(refs.popup) {
            if focus.get().is_none_or(|focused| focused == refs.list) {
                focus.set(refs.input, FocusCause::Navigated);
            }
            return;
        }
    }
    // An outside click may have closed the popup already: the focus still goes back.
    if focus.get().is_none() {
        focus.set(refs.button, FocusCause::Navigated);
    }
    if open {
        commands.entity(root).insert(ComboState::Closed);
    }
}

/// Gives the focus on to the search box if a click put it on the list.
fn on_list_focus(
    gained: On<FocusGained>,
    parts: Query<&FilterPart>,
    combos: Query<(&FilterComboRefs, &ComboState)>,
    mut focus: ResMut<InputFocus>,
) {
    let Ok(&FilterPart(root)) = parts.get(gained.event().entity) else {
        return;
    };
    if let Ok((refs, &ComboState::Open)) = combos.get(root)
        && focus.get() == Some(refs.list)
    {
        focus.set(refs.input, FocusCause::Navigated);
    }
}

/// Closes an open combo when the left button is released outside of it.
///
/// Inside is the button, which toggles the popup itself, and the popup: the search box and the
/// zero-results line always, and the list only for a selection of many, so that a pick there
/// keeps the popup open for the next one.
fn close_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combos: Query<(Entity, &ComboState, &FilterComboRefs), With<FilterCombo>>,
    hovered: Query<&Hovered>,
    scroll_bars: Query<(&PopupScrollBar, &Hovered)>,
    selections: SelectionParam,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if !mouse_button.just_released(MouseButton::Left) {
        return;
    }
    let over = |entity: Entity| hovered.get(entity).is_ok_and(Hovered::get);
    for (root, &state, refs) in &combos {
        if state != ComboState::Open {
            continue;
        }
        let multiple = selections
            .get(refs.list)
            .is_some_and(SelectionValue::allow_multiple);
        let inside = over(refs.popup)
            && (multiple || !over(refs.list) || popup_scroll_bar_hovered(&scroll_bars, refs.list));
        if popup::closes_on_outside_release(true, over(refs.button), inside) {
            if let Ok((mut background, mut border)) = buttons.get_mut(refs.button) {
                popup::apply_look(ButtonLook::Resting, &theme, &mut background, &mut border);
            }
            commands.entity(root).insert(ComboState::Closed);
        }
    }
}

fn handle_button_mouse_events(
    trigger: On<MouseEvent>,
    combos: Query<(&FilterComboRefs, &ComboState)>,
    mut buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    theme: Res<Theme>,
) {
    let Ok((refs, state)) = combos.get(trigger.event().event_target()) else {
        return;
    };
    let Ok((mut background, mut border)) = buttons.get_mut(refs.button) else {
        return;
    };
    if let Some(look) = popup::button_look(trigger.event().kind, *state == ComboState::Open) {
        popup::apply_look(look, &theme, &mut background, &mut border);
    }
}

/// Puts the popup of each open combo under its button, as wide as the button at least, kept
/// inside the window and above the button when it would run off the bottom.
fn place_open_popups(
    viewport: UiViewport,
    combos: Query<(&ComboState, &FilterComboRefs)>,
    buttons: Query<(&ComputedNode, &UiGlobalTransform), With<ComboButton>>,
    mut popups: Query<(&mut Node, &ComputedNode), With<Float>>,
) {
    for (state, refs) in &combos {
        if *state != ComboState::Open {
            continue;
        }
        let (Ok((button_computed, button_transform)), Ok((mut node, popup_computed))) =
            (buttons.get(refs.button), popups.get_mut(refs.popup))
        else {
            continue;
        };
        let Some(target) = viewport.size_of(refs.button) else {
            continue;
        };
        let scale = button_computed.inverse_scale_factor();
        let button_size = button_computed.size() * scale;
        let button_center = button_transform.translation * scale;
        let popup_size = popup_computed.size() * popup_computed.inverse_scale_factor();
        let left = Val::Px(keep_in_window_x(
            button_center.x - button_size.x / 2.0,
            popup_size.x,
            target.x,
        ));
        let fit = fit_vertically(
            button_center.y - button_size.y / 2.0,
            button_center.y + button_size.y / 2.0,
            0.0,
            popup_size.y,
            target.y,
        );
        let (top, max_height, min_width) = (
            Val::Px(fit.top),
            Val::Px(fit.max_height),
            Val::Px(button_size.x),
        );
        if node.left != left
            || node.top != top
            || node.max_height != max_height
            || node.min_width != min_width
        {
            node.left = left;
            node.top = top;
            node.max_height = max_height;
            node.min_width = min_width;
        }
    }
}

/// Scrolls the list just far enough to show the active option and its ring, and keeps the
/// scroll position inside the contents after they shrank. Only the list moves, never a scroll
/// area around the combo.
fn keep_active_in_view(
    mut combos: Query<(&FilterComboRefs, &ComboState, &mut FilterState)>,
    nodes: Query<(&ComputedNode, &UiGlobalTransform)>,
    mut lists: Query<&mut ScrollPosition, With<FilterList>>,
) {
    for (refs, &state, mut filter) in &mut combos {
        if filter.settle == 0 || state != ComboState::Open {
            continue;
        }
        filter.settle -= 1;
        let (Ok((list_computed, list_transform)), Ok(mut scroll)) =
            (nodes.get(refs.list), lists.get_mut(refs.list))
        else {
            continue;
        };
        let scale = list_computed.inverse_scale_factor();
        let list_size = list_computed.size() * scale;
        let max = ((list_computed.content_size() - list_computed.size()) * scale)
            .max(Vec2::ZERO)
            .y;
        let mut position = scroll.0.y.clamp(0.0, max);
        if let Some(active) = filter.active
            && let Ok((row_computed, row_transform)) = nodes.get(active.view)
        {
            let row_scale = row_computed.inverse_scale_factor();
            let row_size = row_computed.size() * row_scale;
            let row_top = row_size
                .y
                .mul_add(-0.5, row_transform.translation.y * row_scale);
            let list_top = list_size
                .y
                .mul_add(-0.5, list_transform.translation.y * scale);
            // The ring is shown with the row, and so is the list's padding beside the first
            // and last: both count as part of it (the padding is wider than the ring).
            let room = POPUP_PADDING.max(FOCUS_RING_REACH);
            let top = row_top - list_top + position - room;
            let bottom = room.mul_add(2.0, top + row_size.y);
            if top < position {
                position = top.clamp(0.0, max);
            } else if bottom > position + list_size.y {
                position = (bottom - list_size.y).clamp(0.0, max);
            }
        }
        if (scroll.0.y - position).abs() > f32::EPSILON {
            scroll.0.y = position;
        }
    }
}

/// Keeps the button and the search box of each combo in step for assistive technology: both
/// are combo boxes controlling the list; the box has the query as its value and the active
/// option as its active descendant.
fn sync_accessibility(
    combos: Query<(
        Entity,
        &FilterComboRefs,
        &ComboState,
        &FilterState,
        &FilterCombo,
    )>,
    selections: SelectionParam,
    items: Query<&Item<String>>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    for (combo, refs, &state, filter, config) in &combos {
        let open = state == ComboState::Open;
        if let Ok(mut node) = nodes.get_mut(refs.button) {
            let value = selections
                .get(combo)
                .map_or_else(String::new, |selection| selected_text(selection, &items));
            sync_role(&mut node, Role::ComboBox);
            sync_value(&mut node, &value);
            sync_controls(&mut node, refs.list);
            sync_name(&mut node, config.name.as_deref());
        }
        if let Ok(mut node) = nodes.get_mut(refs.input) {
            sync_role(&mut node, Role::ComboBox);
            sync_value(&mut node, &filter.query);
            sync_controls(&mut node, refs.list);
            if node.is_expanded() != Some(open) {
                popup::set_expanded(&mut node, open);
            }
            let active = if open {
                filter.active.map(|active| node_id(active.view))
            } else {
                None
            };
            if node.active_descendant() != active {
                match active {
                    Some(id) => node.set_active_descendant(id),
                    None => node.clear_active_descendant(),
                }
            }
        }
    }
}

/// Sets the label of a node to `name`, or clears it, writing only when it differs.
fn sync_name(node: &mut Mut<AccessibilityNode>, name: Option<&str>) {
    match name {
        Some(name) => sync_label(node, name),
        None => {
            if node.label().is_some() {
                node.clear_label();
            }
        }
    }
}

/// Updates the button label and the list's selection when the selection of a combo (own or
/// shared) or its placeholder changes.
fn selection_change(
    changed: Query<Entity, Changed<Selection>>,
    fresh: Query<Entity, Added<FilterComboRefs>>,
    placeholders: Query<Entity, Changed<ComboPlaceholder>>,
    combos: Query<(Entity, &Selection), With<FilterCombo>>,
    mut commands: Commands,
) {
    for (combo, selection) in &combos {
        let selection_changed = changed.iter().any(|changed| match selection {
            Selection::Ref(entity) => changed == *entity || changed == combo,
            Selection::Value(_) => changed == combo,
        });
        if selection_changed || fresh.contains(combo) || placeholders.contains(combo) {
            commands.run_system_cached_with(update_selection, combo);
        }
    }
}

/// Points the list at the selection of the combo and shows the selected items on the button.
fn update_selection(
    combo: In<Entity>,
    selections: SelectionParam,
    refs: Query<&FilterComboRefs>,
    button_refs: Query<&PushButtonRefs>,
    items: Query<&Item<String>>,
    placeholders: Query<&ComboPlaceholder>,
    mut labels: Query<&mut Text, With<PushButtonLabel>>,
    mut commands: Commands,
) {
    let combo = combo.0;
    let (Ok(refs), Some((selection_ref, selection))) =
        (refs.get(combo), selections.get_with_ref(combo))
    else {
        return;
    };
    if selections
        .get_ref(refs.list)
        .is_none_or(|existing| existing != selection_ref)
    {
        commands
            .entity(refs.list)
            .insert(Selection::reference(selection_ref));
    }
    let Ok(button) = button_refs.get(refs.button) else {
        return;
    };
    let Ok(mut text) = labels.get_mut(button.label()) else {
        return;
    };
    text.0 = if selection.is_empty() {
        placeholders
            .get(combo)
            .map_or_else(|_| PLACEHOLDER_TEXT.to_string(), |p| p.0.clone())
    } else {
        selected_text(selection, &items)
    };
}

/// Draws the ring on the active option and takes it off the one that is not any more.
fn style_active_options(
    added: Query<Entity, Added<ActiveOption>>,
    active: Query<Entity, With<ActiveOption>>,
    mut removed: RemovedComponents<ActiveOption>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let ring = Outline {
        width: Val::Px(FOCUS_RING_WIDTH),
        offset: Val::Px(FOCUS_RING_OFFSET),
        color: theme.colors().focus_ring,
    };
    if theme.is_changed() {
        for row in &active {
            commands.entity(row).insert(ring);
        }
    } else {
        for row in &added {
            commands.entity(row).insert(ring);
        }
    }
    for row in removed.read() {
        if !active.contains(row)
            && let Ok(mut row) = commands.get_entity(row)
        {
            row.remove::<Outline>();
        }
    }
}

fn theme_filter_combos(
    combo: In<Option<Entity>>,
    theme: Res<Theme>,
    combos: Query<&FilterComboRefs>,
    mut popups: Query<(&mut Node, &mut BackgroundColor, &mut BorderColor), Without<FilterEmpty>>,
    mut empties: Query<(&mut TextColor, &mut TextFont), With<FilterEmpty>>,
) {
    let mut theme_one = |refs: &FilterComboRefs| {
        if let Ok((mut node, mut background, mut border)) = popups.get_mut(refs.popup) {
            node.border = UiRect::all(theme.border_thickness);
            background.0 = theme.colors().background_primary;
            border.set_all(theme.colors().border);
        }
        if let Ok((mut color, mut font)) = empties.get_mut(refs.empty) {
            color.0 = theme.colors().text_secondary;
            font.font = theme.body_font.clone().into();
            font.font_size = Theme::font_size(theme.font_size_secondary);
        }
    };
    match combo.0 {
        Some(combo) => {
            if let Ok(refs) = combos.get(combo) {
                theme_one(refs);
            }
        }
        None => combos.iter().for_each(theme_one),
    }
}
