//! `ColorPaletteCombo`: a [`ColorPalette`] inside a combo-style popup.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusCause, FocusLost, FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use bevy::window::PrimaryWindow;
use itertools::Either;

use crate::color_palette::ColorPalette;
use crate::color_push_button::{ColorPushButton, ColorSquare, ColorSquareRef};
use crate::float::Float;
use crate::focus::{
    AccessibilitySync, Unreachable, clear_label, popup_focus_lost, popup_key_closes, sync_controls,
    sync_role, sync_value,
};
use crate::list::{ItemView, ListLayout};
use crate::push_button::NoDefaultHover;
use crate::scroll_area::ScrollArea;
use crate::select::{Item, Selection, SelectionParam, SelectionValue};
use crate::tailwind::color_name;
use crate::theme::Theme;
use crate::{MouseEvent, MouseKind, color_push_button};

/// Registers the systems and observers of [`ColorPaletteCombo`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug, Default)]
pub struct ColorPaletteComboPlugin;

impl Plugin for ColorPaletteComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_floats)
                    .run_if(resource_changed::<Theme>),
                close_combos_on_outside_click,
                on_selection_change,
            ),
        )
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
        .add_observer(on_insert_combo)
        .add_observer(on_remove_combo)
        .add_observer(on_insert_state)
        .add_observer(close_on_swatch_click);
    }
}

#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
// #[component(on_add=Self::on_insert)]
/// A button that opens a [`ColorPalette`] in a popup.
///
/// Opening the popup moves the keyboard focus to its palette, so the arrow keys, Enter and
/// Space work on it. Escape closes it, as does the focus leaving the palette (Tab away, a
/// click elsewhere); the focus then goes back to the button, unless it moved to another widget.
///
/// To assistive technology the button is a combo box ([`Role::ComboBox`]) without a label:
/// its value is the name of the selected colour (empty for none), it is expanded while the
/// popup is open, and it controls the popup's palette.
#[require(
    Node {
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Name::new("ColorPaletteCombo"),
)]
pub struct ColorPaletteCombo;

/// Marks the button of a [`ColorPaletteCombo`].
///
/// Its [`Hovered`] says whether the pointer is on the combo, for the outside click. A
/// [`ColorPushButton`] is named after its colour; this one is not, since its value says the
/// colour.
#[derive(Component, Debug, Copy, Clone)]
#[require(Hovered)]
pub(crate) struct ColorPaletteComboButton;

/// Marks the popup of a [`ColorPaletteCombo`].
///
/// Its [`Hovered`] says whether the pointer is on the popup, for the outside click.
#[derive(Component, Debug, Copy, Clone)]
#[require(Hovered)]
struct ColorPaletteComboFloat;

/// Builds a [`ColorPaletteCombo`]: its button and its popup, both children of the combo.
///
/// The popup is a [`Float`], placed in the viewport rather than in the combo, so nothing
/// around the combo (a scroll area, a card) clips it or draws over it.
///
/// The popup is named after its combo, `<combo name> popup`, so that a scenario
/// or a query finds it by name whatever the order the combos were spawned in.
/// The name is taken now: renaming the combo later does not rename its popup.
///
/// The popup's [`ColorPalette`] refers to the combo's selection from the start, so that
/// it shows and changes it without waiting for a change of the selection. A combo that
/// has no [`Selection`] yet gets the reference when it is given one, see
/// [`on_selection_change`].
fn on_insert_combo(
    t: On<Insert<ColorPaletteCombo>>,
    mut commands: Commands,
    list_layouts: Query<&ListLayout>,
    selections: SelectionParam,
    names: Query<&Name>,
) {
    let combo = t.event().event_target();
    let popup_name = popup_name(names.get(combo).ok());

    let mut button_id = Entity::PLACEHOLDER;
    let mut float_id = Entity::PLACEHOLDER;
    // Reserved now so that the combo can keep it; the scroll area inserts it as its contents.
    let palette_id = commands.spawn_empty().id();

    let list_layout = list_layouts.get(combo).ok().copied();
    let selection_ref = selections.get_ref(combo);
    commands.entity(combo).with_children(|parent| {
        button_id = parent
            .spawn((
                ColorPaletteComboButton,
                ColorPushButton,
                NoDefaultHover,
                // allow mouse events to pass through to parent
                Pickable {
                    should_block_lower: false,
                    ..default()
                },
            ))
            .id();
        float_id = parent
            .spawn((
                Float,
                ColorPaletteComboFloat,
                popup_name,
                Node {
                    left: Val::Px(0.0),
                    top: Val::Px(40.0),
                    position_type: PositionType::Absolute,
                    height: Val::Px(250.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor::all(Color::BLACK),
                Visibility::Hidden,
                ScrollArea::bundle(
                    move |parent| {
                        let contents = parent.target_entity();
                        let mut palette = parent.world_mut().entity_mut(palette_id);
                        palette.insert((
                            ChildOf(contents),
                            Pickable {
                                should_block_lower: false,
                                ..default()
                            },
                            ColorPalette,
                            list_layout.unwrap_or(ListLayout::VerticalGrid { columns: 6 }),
                        ));
                        if let Some(selection_ref) = selection_ref {
                            palette.insert(Selection::reference(selection_ref));
                        }
                    },
                    true,
                    false,
                ),
            ))
            .id();
    });

    let sub_refs = SubRefs {
        parent: combo,
        button: button_id,
        float_id,
        palette: palette_id,
    };
    commands.entity(combo).insert((State::Closed, sub_refs));

    commands
        .entity(palette_id)
        .insert(sub_refs)
        .observe(close_on_key)
        .observe(close_on_focus_lost);

    commands
        .entity(button_id)
        .insert(sub_refs)
        .observe(handle_combo_button_mouse_events)
        .observe(toggle_on_activate);
}

/// The name of the popup of a combo called `combo_name`.
fn popup_name(combo_name: Option<&Name>) -> Name {
    Name::new(format!(
        "{} popup",
        combo_name.map_or("ColorPaletteCombo", Name::as_str)
    ))
}

/// Closes the combo on the keyboard, with the focus on its palette.
fn close_on_key(
    input: On<FocusedInput<KeyboardInput>>,
    sub_refs: Query<&SubRefs>,
    states: Query<&State>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let Ok(sub_refs) = sub_refs.get(input.event_target()) else {
        return;
    };
    if states.get(sub_refs.parent) == Ok(&State::Open)
        && popup_key_closes(&input.event().input, selections.get(sub_refs.parent))
    {
        commands.entity(sub_refs.parent).insert(State::Closed);
    }
}

/// Closes the combo when the focus leaves its palette.
fn close_on_focus_lost(
    lost: On<FocusLost>,
    sub_refs: Query<&SubRefs>,
    states: Query<&State>,
    hovered: Query<&Hovered, With<ColorPaletteComboFloat>>,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    // The event bubbles: only the palette itself losing the focus counts.
    let Ok(sub_refs) = sub_refs.get(lost.event().entity) else {
        return;
    };
    let Ok(&state) = states.get(sub_refs.parent) else {
        return;
    };
    if sub_refs.palette != lost.event().entity {
        return;
    }
    let popup_hovered = hovered.get(sub_refs.float_id).is_ok_and(Hovered::get);
    if popup_focus_lost(
        &mut focus,
        state == State::Open,
        sub_refs.button,
        sub_refs.palette,
        popup_hovered,
    ) {
        commands.entity(sub_refs.parent).insert(State::Closed);
    }
}

fn on_remove_combo(
    t: On<Remove<ColorPaletteCombo>>,
    sub_refs: Query<&SubRefs>,
    mut commands: Commands,
) {
    // The popup is a child of the combo, so it goes with a despawned combo by itself.
    if let Ok(sub_refs) = sub_refs.get(t.event().event_target()) {
        commands.entity(sub_refs.float_id).try_despawn();
    }
}

#[derive(Component, Debug, Copy, Clone)]
struct SubRefs {
    parent: Entity,
    button: Entity,
    float_id: Entity,
    /// The popup's [`ColorPalette`].
    palette: Entity,
}

/// Whether the popup of a [`ColorPaletteCombo`] is showing.
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub enum State {
    /// The popup is showing.
    Open,
    /// The popup is hidden.
    Closed,
}

impl State {
    /// The other state.
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }
}

fn on_insert_state(
    t: On<Insert<State>>,
    state: Query<(&State, &SubRefs)>,
    mut accessibility_nodes: Query<&mut AccessibilityNode>,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let Ok((&state, &sub_refs)) = state.get(t.event().event_target()) else {
        return;
    };
    // A closed popup's palette is not a tab stop.
    match state {
        State::Open => commands.entity(sub_refs.palette).remove::<Unreachable>(),
        State::Closed => commands.entity(sub_refs.palette).insert(Unreachable),
    };
    match state {
        State::Open => focus.set(sub_refs.palette, FocusCause::Navigated),
        // Whoever closed the popup, the focus does not stay on a hidden palette.
        State::Closed if focus.get() == Some(sub_refs.palette) => {
            focus.set(sub_refs.button, FocusCause::Navigated);
        }
        State::Closed => {}
    }

    let visibility = match state {
        State::Open => Visibility::Visible,
        State::Closed => Visibility::Hidden,
    };
    commands.entity(sub_refs.float_id).insert(visibility);
    if let Ok(mut node) = accessibility_nodes.get_mut(sub_refs.button) {
        node.set_expanded(state == State::Open);
    }
    if state == State::Open {
        commands.run_system_cached_with(position_float, sub_refs);
    }
}

/// Places the popup of a combo under its button, centred on it, in viewport coordinates.
///
/// The popup is moved in until it is 10 pixels from the window's left and right
/// edges, and goes above the button instead of under it when it would run off the bottom. Only
/// the button's position and the popup's laid-out size are read, so the first opening is placed
/// like any other.
fn position_float(
    sub_refs: In<SubRefs>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut dims: Query<(&mut Node, &ComputedNode), With<ColorPaletteComboFloat>>,
    parent_dims: Query<
        (&ComputedNode, &UiGlobalTransform),
        (
            With<ColorPaletteComboButton>,
            Without<ColorPaletteComboFloat>,
        ),
    >,
) {
    const MARGIN: f32 = 10.0;
    const BUTTON_FLOAT_DIST: f32 = 3.0;

    let (Ok((mut node, float_computed_node)), Ok((button_computed_node, button_transform))) = (
        dims.get_mut(sub_refs.float_id),
        parent_dims.get(sub_refs.button),
    ) else {
        return;
    };

    let button_size = button_computed_node.size() * button_computed_node.inverse_scale_factor();
    let float_size = float_computed_node.size() * float_computed_node.inverse_scale_factor();
    // `UiGlobalTransform` holds the centre of the node.
    let button_center = button_transform.translation * button_computed_node.inverse_scale_factor();

    // Centred under the button, then pushed back inside the window. A popup wider than the
    // window keeps its left edge on the margin.
    let centered_x = button_center.x - float_size.x / 2.0;
    let max_x = window.width() - MARGIN - float_size.x;
    node.left = Val::Px(centered_x.min(max_x).max(MARGIN));

    let below_y = button_center.y + button_size.y / 2.0 + BUTTON_FLOAT_DIST;
    node.top = Val::Px(if below_y + float_size.y + MARGIN > window.height() {
        button_center.y - button_size.y / 2.0 - BUTTON_FLOAT_DIST - float_size.y
    } else {
        below_y
    });
}

/// Keeps the popup's palette and the button's colour square in step with the combo's
/// selection.
///
/// The palette already refers to the selection the combo had when it was spawned; this
/// points it at the new one when the combo gets or is given another selection.
fn on_selection_change(
    q: Query<Entity, Changed<Selection>>,
    combos: Query<(Entity, &SubRefs), With<ColorPaletteCombo>>,
    color_square_refs: Query<&ColorSquareRef>,
    mut color_squares: Query<&mut BackgroundColor, With<ColorSquare>>,
    selections: SelectionParam,
    items: Query<&Item<Color>>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (color_palette_combo_ref, sub_refs) in combos {
            let Some((selection_ref, selection)) = selections.get_with_ref(color_palette_combo_ref)
            else {
                continue;
            };
            // Its own selection, or the one it refers to.
            if selection_ref != changed_selection_ref
                && color_palette_combo_ref != changed_selection_ref
            {
                continue;
            }

            if selections
                .get_ref(sub_refs.palette)
                .is_none_or(|existing_ref| existing_ref != selection_ref)
            {
                // Selection id has changed -> propagate to the palette
                commands
                    .entity(sub_refs.palette)
                    .insert(Selection::reference(selection_ref));
            }

            let Ok(ColorSquareRef(color_square_id)) = color_square_refs.get(sub_refs.button) else {
                continue;
            };
            let Ok(mut color_square_background) = color_squares.get_mut(*color_square_id) else {
                continue;
            };

            assert!(!selection.allow_multiple(), "Multi selection not supported");
            let color = selection
                .single()
                .and_then(|selected| items.get(selected).ok())
                .map_or(Color::Srgba(Srgba::NONE), |item| item.data);
            color_square_background.0 = color;
            commands.run_system_cached_with(color_push_button::update_theme, Some(sub_refs.button));
        }
    }
}

/// Keeps the button of each combo a combo box to assistive technology, with the name of the
/// selected colour as its value and the popup's palette as what it controls. Bevy makes a
/// node a button again when its `Button` changes, so this runs after it.
fn sync_accessibility(
    combos: Query<(Entity, &SubRefs), With<ColorPaletteCombo>>,
    selections: SelectionParam,
    items: Query<&Item<Color>>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    for (combo, sub_refs) in &combos {
        let Ok(mut node) = nodes.get_mut(sub_refs.button) else {
            continue;
        };
        let value = selections
            .get(combo)
            .and_then(SelectionValue::single)
            .and_then(|selected| items.get(selected).ok())
            .map_or_else(String::new, |item| color_name(item.data));
        sync_role(&mut node, Role::ComboBox);
        clear_label(&mut node);
        sync_value(&mut node, &value);
        sync_controls(&mut node, sub_refs.palette);
    }
}

/// Closes an open combo when the left button is released outside of it.
///
/// Inside is the combo's button, which opens and closes the popup itself on
/// [`Activate`], and its popup, whatever the selection: a release on the popup's
/// scroll bar or between its swatches leaves it open, and a pick closes it on its own,
/// see [`close_on_swatch_click`]. Whether the pointer is on them is their [`Hovered`],
/// which comes from picking and so works whatever the camera renders to, a window or
/// an image.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combos: Query<(Entity, &State, &SubRefs), With<ColorPaletteCombo>>,
    mut combo_buttons: Query<
        (&Hovered, &mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    floats: Query<&Hovered, With<ColorPaletteComboFloat>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if mouse_button.just_released(MouseButton::Left) {
        for (combo_id, &combo_state, subs) in combos.iter() {
            let Ok((over_button, mut background_color, mut border_color)) =
                combo_buttons.get_mut(subs.button)
            else {
                continue;
            };
            let over_float = floats.get(subs.float_id).is_ok_and(Hovered::get);
            if combo_state == State::Open && !over_button.get() && !over_float {
                close(
                    combo_id,
                    &mut background_color,
                    &mut border_color,
                    &theme,
                    &mut commands,
                );
            }
        }
    }
}

/// Closes the popup of a combo and puts its button back to the resting colours.
fn close(
    combo: Entity,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
    theme: &Theme,
    commands: &mut Commands,
) {
    *background_color = BackgroundColor(theme.colors.background_primary);
    *border_color = BorderColor::all(theme.colors.border);
    commands.entity(combo).insert(State::Closed);
}

/// Closes the popup of a combo whose selection holds one item at most when a swatch
/// in it is clicked, left button, whether or not the click changed the selection.
///
/// It is a global observer of [`MouseEvent`]s because a click is a press and a release on
/// the same swatch, which `MouseKind::Click` says and a `Released` cannot (a release alone
/// also ends a press that began elsewhere, a drag of the popup's scroll thumb that ends
/// over a swatch). It acts only on a click whose target is a [`swatch`](ItemView) that
/// belongs to a palette of a combo, so a click on the popup that is not on a swatch (its
/// scroll bar, the gap between swatches) does not close it. A selection of many
/// (`OneOrMore`, `ZeroOrMore`) keeps the popup open for the next pick.
fn close_on_swatch_click(
    t: On<MouseEvent>,
    palettes: Query<&SubRefs, With<ColorPalette>>,
    swatches: Query<&ItemView>,
    states: Query<&State, With<ColorPaletteCombo>>,
    mut combo_buttons: Query<
        (&mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    selections: SelectionParam,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let MouseKind::Click(button) = t.event().kind else {
        return;
    };
    if button != PointerButton::Primary {
        return;
    }
    let Ok(ItemView { list_ref, .. }) = swatches.get(t.event().event_target()) else {
        return;
    };
    let Ok(subs) = palettes.get(*list_ref) else {
        return;
    };
    let Some(selection) = selections.get(subs.parent) else {
        return;
    };
    if selection.allow_multiple() || states.get(subs.parent) != Ok(&State::Open) {
        return;
    }
    let Ok((mut background_color, mut border_color)) = combo_buttons.get_mut(subs.button) else {
        return;
    };
    close(
        subs.parent,
        &mut background_color,
        &mut border_color,
        &theme,
        &mut commands,
    );
}

fn handle_combo_button_mouse_events(
    trigger: On<MouseEvent>,
    mut combo_buttons: Query<
        (&SubRefs, &mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    combos: Query<&State, With<ColorPaletteCombo>>,
    theme: Res<Theme>,
) {
    let Ok((
        SubRefs {
            parent: parent_id, ..
        },
        mut background_color,
        mut border_color,
    )) = combo_buttons.get_mut(trigger.event().event_target())
    else {
        return;
    };
    let Ok(combo_state) = combos.get(*parent_id) else {
        return;
    };

    match (trigger.event().kind, combo_state) {
        // A drag says nothing new about the pointer being on the button, and can arrive
        // after the pointer left it, so it does not restyle the button. A release or a
        // click on a closed combo does not either, so all three share one empty arm.
        (MouseKind::Drag, _) | (MouseKind::Released | MouseKind::Click(_), State::Closed) => {}
        (MouseKind::Over | MouseKind::Pressed, _) | (_, State::Open) => {
            *background_color = BackgroundColor(theme.colors.background_secondary);
            *border_color = BorderColor::all(theme.colors.border_highlight);
        }

        (MouseKind::Out, State::Closed) => {
            *background_color = BackgroundColor(theme.colors.background_primary);
            *border_color = BorderColor::all(theme.colors.border);
        }
    }
}

/// Opens the popup of a [`ColorPaletteCombo`] whose button was activated, or closes it.
fn toggle_on_activate(
    activate: On<Activate>,
    buttons: Query<&SubRefs, With<ColorPaletteComboButton>>,
    states: Query<&State, With<ColorPaletteCombo>>,
    mut commands: Commands,
) {
    let Ok(&SubRefs { parent, .. }) = buttons.get(activate.event_target()) else {
        return;
    };
    let Ok(state) = states.get(parent) else {
        return;
    };
    commands.entity(parent).insert(state.inverse());
}

fn theme_floats(
    float_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut floats: Query<&mut BorderColor, With<ColorPaletteComboFloat>>,
) {
    let it = if let Some(float_id) = float_id.as_ref() {
        Either::Left(floats.get_mut(*float_id).into_iter())
    } else {
        Either::Right(floats.iter_mut())
    };

    for mut border_color in it {
        *border_color = BorderColor::all(theme.colors.border_highlight);
    }
}
