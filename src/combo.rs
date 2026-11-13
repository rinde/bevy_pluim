//! `Combo`: a button that opens a [`List`] in a [`Float`].
//!
//! The popup closes again on a click outside of it. From the keyboard, opening it puts the
//! focus on its list and Escape closes it again.

use bevy::a11y::AccessibilityNode;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusLost, FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::{ComputedNode, UiGlobalTransform, UiSystems};
use bevy::ui_widgets::Activate;
use itertools::Itertools;

use crate::MouseEvent;
use crate::float::{Float, UiViewport, fit_vertically, keep_in_window_x};
use crate::focus::{AccessibilitySync, popup_focus_lost, popup_key_closes};
use crate::glyph_images::{GlyphKey, shared};
use crate::icon::Icon;
use crate::list::{List, NoDefaultItems};
use crate::popup::{self, ButtonLook};
use crate::push_button::{NoDefaultHover, PushButton, PushButtonLabel, PushButtonRefs};
use crate::scroll_area::{
    PopupScrollBar, popup_scroll_bar, popup_scroll_bar_hovered, update_scroll_position,
};
use crate::select::{Item, Selection, SelectionParam, SelectionValue};
use crate::theme::Theme;

/// Registers the systems and observers of [`Combo`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct ComboPlugin;

impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (close_combos_on_outside_click, selection_change))
            .add_systems(
                Update,
                close_on_ancestor_scroll.after(update_scroll_position),
            )
            .add_systems(
                PostUpdate,
                (
                    sync_accessibility.in_set(AccessibilitySync),
                    place_open_lists.before(UiSystems::Layout),
                ),
            )
            .add_observer(on_insert_combo)
            .add_observer(on_insert_placeholder);
    }
}

/// A button that opens a [`List`] in a [`Float`].
///
/// The popup closes again on a click outside of it or when an ancestor scrolls. The button toggles it when
/// it is [`Activate`]d (a click, or Enter or Space on a focused button).
///
/// To assistive technology the button is a combo box ([`Role::ComboBox`](accesskit::Role::ComboBox)) without a
/// label: its value is the text of the selected items joined with ", " (empty for none),
/// it is expanded while the popup is open, and it controls the popup's list.
///
/// Opening the popup moves the keyboard focus to its list, so the arrow keys, Enter and
/// Space work on it. Escape closes it, as does the focus leaving the list (Tab away, a click
/// elsewhere); the focus then goes back to the button, unless it moved to another widget.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
#[component(on_add=Self::on_add)]
#[require(
    Node {
        width: Val::Px(200.0),
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Name::new("Combo"),
)]
pub struct Combo;

/// Marks the button of a [`Combo`].
///
/// Its [`Hovered`] says whether the pointer is on the combo, for the outside click.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Name::new("ComboButton"), Hovered)]
pub struct ComboButton;

/// The text a [`Combo`]'s button shows while nothing is selected, "Select.." by default.
///
/// Put it on the combo, with it or later; inserting it again changes the text of a live
/// combo. To a screen reader a combo with no selection still has an empty value, not this
/// text. Removing it does not change the button; insert `ComboPlaceholder::default()` to
/// restore the default.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ComboPlaceholder(pub String);

impl Default for ComboPlaceholder {
    fn default() -> Self {
        Self(PLACEHOLDER_TEXT.to_string())
    }
}

/// The parts of a [`Combo`], on the combo itself. Only the widget builds it.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ComboRefs {
    button_id: Entity,
    list_id: Entity,
}

impl ComboRefs {
    /// The button that opens and closes the popup.
    #[must_use]
    pub const fn button(&self) -> Entity {
        self.button_id
    }

    /// The list inside the popup.
    #[must_use]
    pub const fn list(&self) -> Entity {
        self.list_id
    }
}

/// Marks the list inside the popup of a [`Combo`].
///
/// Its [`Hovered`] says whether the pointer is on the popup, for the outside click.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Name::new("ComboList"), Hovered)]
pub struct ComboList;

/// Whether the popup of a [`Combo`] is showing.
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert=Self::on_insert)]
pub enum ComboState {
    /// The popup is showing.
    Open,
    /// The popup is hidden.
    Closed,
}

impl ComboState {
    /// The other state.
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let entity = world.entity(context.entity);
        // Not a live `Combo` (no `ComboRefs`): there is no popup to show or hide.
        let (Some(&state), Some(&ComboRefs { list_id, button_id })) =
            (entity.get::<Self>(), entity.get::<ComboRefs>())
        else {
            return;
        };

        let open = state == Self::Open;
        popup::show_popup(&mut world.commands(), open, list_id, list_id);
        if let Some(mut focus) = world.get_resource_mut::<InputFocus>() {
            popup::move_focus(&mut focus, open, list_id, button_id);
        }
        if let Some(mut node) = world.get_mut::<AccessibilityNode>(button_id) {
            popup::set_expanded(&mut node, open);
        }
    }
}

pub(crate) const PLACEHOLDER_TEXT: &str = "Select..";

/// The side of the chevron image, in logical pixels.
pub(crate) const CHEVRON_SIZE: f32 = 16.0;

/// A white-on-transparent 16 px triangle pointing down, tinted by its [`Icon`].
pub(crate) fn chevron() -> Image {
    const SIZE: u32 = 16;
    let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 5..11u32 {
        let half_width = 6 - (y - 5);
        for x in (8 - half_width)..(8 + half_width) {
            let at = ((y * SIZE + x) * 4) as usize;
            if let Some(pixel) = data.get_mut(at..at + 4) {
                pixel.copy_from_slice(&[255; 4]);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

impl Combo {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let mut button_id = Entity::PLACEHOLDER;
        let mut list_id = Entity::PLACEHOLDER;
        let placeholder = world
            .get::<ComboPlaceholder>(context.entity)
            .map_or_else(|| PLACEHOLDER_TEXT.to_string(), |p| p.0.clone());
        let chevron = shared(&mut world, GlyphKey::Chevron, chevron);
        world.commands().entity(context.entity).with_children(|x| {
            button_id = x
                .spawn((
                    PushButton::from_label(placeholder),
                    ComboButton,
                    NoDefaultHover,
                    // allow mouse events to pass through to parent
                    Pickable {
                        should_block_lower: false,
                        ..default()
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Icon::bundle(chevron, CHEVRON_SIZE),
                        Name::new("Chevron"),
                        Pickable::IGNORE,
                    ));
                })
                .id();

            x.spawn(Node::DEFAULT).with_children(|spawner| {
                list_id = spawner
                    .spawn((
                        List,
                        Float,
                        // A popup floats over what is below it, so it takes the pointer from
                        // there: a `List` lets it through, and a click on an item would
                        // click the item beneath it as well.
                        Pickable::default(),
                        Visibility::Hidden,
                        ComboList,
                    ))
                    .id();
            });
        });

        // The list scrolls itself when it is capped to the window: it gets a bar.
        world.commands().spawn(popup_scroll_bar(list_id));

        let combo_id = context.entity;
        world
            .commands()
            .entity(list_id)
            .observe(close_on_key(combo_id))
            .observe(close_on_focus_lost(combo_id));
        world
            .commands()
            .entity(combo_id)
            .insert((ComboState::Closed, ComboRefs { button_id, list_id }))
            .observe(handle_combo_button_mouse_events);
        world.commands().entity(button_id).observe(
            move |_: On<Activate>, states: Query<&ComboState>, mut commands: Commands| {
                if let Ok(state) = states.get(combo_id) {
                    commands.entity(combo_id).insert(state.inverse());
                }
            },
        );
    }
}

/// An observer for the popup's list that closes the combo on the keyboard.
fn close_on_key(
    combo_id: Entity,
) -> impl Fn(On<FocusedInput<KeyboardInput>>, Query<&ComboState>, SelectionParam, Commands) {
    move |input, states, selections, mut commands| {
        if states.get(combo_id) == Ok(&ComboState::Open)
            && popup_key_closes(&input.event().input, selections.get(combo_id))
        {
            commands.entity(combo_id).insert(ComboState::Closed);
        }
    }
}

/// An observer for the popup's list that closes the combo when the focus leaves the list.
fn close_on_focus_lost(
    combo_id: Entity,
) -> impl Fn(
    On<FocusLost>,
    Query<(&ComboState, &ComboRefs)>,
    Query<&Hovered, With<ComboList>>,
    ResMut<InputFocus>,
    Commands,
) {
    move |lost, states, hovered, mut focus, mut commands| {
        let Ok((&state, sub_refs)) = states.get(combo_id) else {
            return;
        };
        // The event bubbles: only the list itself losing the focus counts.
        if lost.event().entity != sub_refs.list_id {
            return;
        }
        let popup_hovered = hovered.get(sub_refs.list_id).is_ok_and(Hovered::get);
        if popup_focus_lost(
            &mut focus,
            state == ComboState::Open,
            sub_refs.button_id,
            sub_refs.list_id,
            popup_hovered,
        ) {
            commands.entity(combo_id).insert(ComboState::Closed);
        }
    }
}

/// Puts the list of each open combo right under its button, its left edge on the button's.
///
/// The list is moved in until it is 10 pixels from the window's left and right edges, and goes
/// above the button when it would run off the bottom, as a palette combo's popup does.
///
/// The list is a [`Float`], placed in the viewport, so it is moved here from the button's
/// last laid-out position, in logical pixels, and follows the button while open (a scroll area
/// around the combo scrolling, say).
fn close_on_ancestor_scroll(
    combos: Query<(Entity, &ComboState), With<Combo>>,
    parents: Query<&ChildOf>,
    scrolled: Query<(), Changed<ScrollPosition>>,
    mut commands: Commands,
) {
    for (entity, state) in &combos {
        if *state == ComboState::Open
            && parents
                .iter_ancestors(entity)
                .any(|ancestor| scrolled.contains(ancestor))
        {
            commands.entity(entity).insert(ComboState::Closed);
        }
    }
}

fn place_open_lists(
    viewport: UiViewport,
    combos: Query<(&ComboState, &ComboRefs)>,
    buttons: Query<(&ComputedNode, &UiGlobalTransform), With<ComboButton>>,
    mut lists: Query<(&mut Node, &ComputedNode), With<ComboList>>,
) {
    for (state, sub_refs) in &combos {
        if *state != ComboState::Open {
            continue;
        }
        let (Ok((button_computed, button_transform)), Ok((mut node, list_computed))) = (
            buttons.get(sub_refs.button_id),
            lists.get_mut(sub_refs.list_id),
        ) else {
            continue;
        };
        let Some(target) = viewport.size_of(sub_refs.button_id) else {
            continue;
        };
        let scale = button_computed.inverse_scale_factor();
        let button_size = button_computed.size() * scale;
        // `UiGlobalTransform` holds the centre of the node.
        let button_center = button_transform.translation * scale;
        let list_size = list_computed.size() * list_computed.inverse_scale_factor();
        let left = Val::Px(keep_in_window_x(
            button_center.x - button_size.x / 2.0,
            list_size.x,
            target.x,
        ));
        let fit = fit_vertically(
            button_center.y - button_size.y / 2.0,
            button_center.y + button_size.y / 2.0,
            0.0,
            list_size.y,
            target.y,
        );
        let top = Val::Px(fit.top);
        let max_height = Val::Px(fit.max_height);
        if node.left != left
            || node.top != top
            || node.max_height != max_height
            || node.overflow.y != OverflowAxis::Scroll
        {
            node.left = left;
            node.top = top;
            // A list taller than the window scrolls with the wheel instead of running off it.
            node.max_height = max_height;
            node.overflow.y = OverflowAxis::Scroll;
        }
    }
}

fn on_insert_combo(t: On<Insert<Combo>>, selections: SelectionParam, mut commands: Commands) {
    let combo_ref = t.event().event_target();
    let Some(combo_selection) = selections.get_ref(combo_ref) else {
        return;
    };
    commands.run_system_cached_with(update_selection, (combo_selection, combo_ref));
}

fn on_insert_placeholder(
    t: On<Insert<ComboPlaceholder>>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let combo_ref = t.event().event_target();
    let Some(combo_selection) = selections.get_ref(combo_ref) else {
        return;
    };
    commands.run_system_cached_with(update_selection, (combo_selection, combo_ref));
}

fn selection_change(
    q: Query<Entity, Changed<Selection>>,
    combos: Query<(Entity, &Selection), (With<Combo>, Without<NoDefaultItems>)>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (combo_ref, combo_selection) in combos {
            let is_changed = match combo_selection {
                Selection::Ref(entity) => changed_selection_ref == *entity,
                Selection::Value(_selection) => changed_selection_ref == combo_ref,
            };
            if is_changed {
                commands
                    .run_system_cached_with(update_selection, (changed_selection_ref, combo_ref));
            }
        }
    }
}

fn update_selection(
    input: In<(Entity, Entity)>,
    selections: SelectionParam,
    combo_sub_refs: Query<&ComboRefs>,
    pblr: Query<&PushButtonRefs>,
    items: Query<&Item<String>>,
    placeholders: Query<&ComboPlaceholder>,
    mut labels: Query<&mut Text, With<PushButtonLabel>>,
    mut commands: Commands,
) {
    let (selection_ref, combo_ref) = *input;
    let (Ok(sub_refs), Some(selection)) =
        (combo_sub_refs.get(combo_ref), selections.get(selection_ref))
    else {
        return;
    };

    if selections
        .get_ref(sub_refs.list_id)
        .is_none_or(|existing_ref| existing_ref != selection_ref)
    {
        // Selection id has changed -> propagate to the List
        commands
            .entity(sub_refs.list_id)
            .insert(Selection::reference(selection_ref));
    }

    let Ok(refs) = pblr.get(sub_refs.button_id) else {
        return;
    };
    let Ok(mut text) = labels.get_mut(refs.label()) else {
        return;
    };
    let label = if selection.is_empty() {
        placeholders
            .get(combo_ref)
            .map_or_else(|_| PLACEHOLDER_TEXT.to_string(), |p| p.0.clone())
    } else {
        selected_text(selection, &items)
    };
    text.0 = label;
}

/// The text of the items in `selection`, joined with ", ".
pub(crate) fn selected_text(selection: &SelectionValue, items: &Query<&Item<String>>) -> String {
    selection
        .iter()
        .map(|item| items.get(item).map_or("", |item| item.data.as_str()))
        .join(", ")
}

/// Keeps the button of each combo a combo box to assistive technology, with the text of the
/// selected items as its value and the popup's list as what it controls. Neither the chevron
/// nor the placeholder is part of it: with nothing selected the value is empty. Bevy makes a
/// node a button again when its `Button` changes, so this runs after it.
fn sync_accessibility(
    combos: Query<(Entity, &ComboRefs), With<Combo>>,
    selections: SelectionParam,
    items: Query<&Item<String>>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    for (combo, sub_refs) in &combos {
        let Ok(mut node) = nodes.get_mut(sub_refs.button_id) else {
            continue;
        };
        let value = selections
            .get(combo)
            .map_or_else(String::new, |selection| selected_text(selection, &items));
        popup::sync_combo_box(&mut node, &value, sub_refs.list_id);
    }
}

/// Closes an open combo when the left button is released outside of it.
///
/// Inside is the combo's button, which opens and closes the popup itself on
/// [`Activate`], and, for a selection that may hold several items, its list, so
/// that a pick there keeps the popup open. Whether the pointer is on them is their
/// [`Hovered`], which comes from picking and so works whatever the camera renders
/// to, a window or an image.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combos: Query<(Entity, &ComboState, &ComboRefs), With<Combo>>,
    selections: SelectionParam,
    mut combo_buttons: Query<(&Hovered, &mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    combo_lists: Query<&Hovered, With<ComboList>>,
    scroll_bars: Query<(&PopupScrollBar, &Hovered)>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if mouse_button.just_released(MouseButton::Left) {
        for (combo_ref, &combo_state, subs) in combos.iter() {
            let Some(selection_value) = selections.get(combo_ref) else {
                continue;
            };
            let Ok((over_button, mut background_color, mut border_color)) =
                combo_buttons.get_mut(subs.button_id)
            else {
                continue;
            };
            let over_list = combo_lists.get(subs.list_id).is_ok_and(Hovered::get);
            // A pick in the list of a selection of many keeps the popup open.
            if popup::closes_on_outside_release(
                combo_state == ComboState::Open,
                over_button.get(),
                (selection_value.allow_multiple() && over_list)
                    || popup_scroll_bar_hovered(&scroll_bars, subs.list_id),
            ) {
                popup::apply_look(
                    ButtonLook::Resting,
                    &theme,
                    &mut background_color,
                    &mut border_color,
                );
                commands.entity(combo_ref).insert(ComboState::Closed);
            }
        }
    }
}

fn handle_combo_button_mouse_events(
    trigger: On<MouseEvent>,
    combos: Query<(&ComboRefs, &ComboState), With<Combo>>,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    theme: Res<Theme>,
) {
    let Ok((ComboRefs { button_id, .. }, combo_state)) = combos.get(trigger.event().event_target())
    else {
        return;
    };
    let Ok((mut background_color, mut border_color)) = combo_buttons.get_mut(*button_id) else {
        return;
    };

    if let Some(look) = popup::button_look(trigger.event().kind, *combo_state == ComboState::Open) {
        popup::apply_look(look, &theme, &mut background_color, &mut border_color);
    }
}
