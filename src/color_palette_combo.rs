//! `ColorPaletteCombo`: a [`ColorPalette`] inside a combo-style popup.

use bevy::a11y::AccessibilityNode;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusLost, FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::picking::pointer::PointerInput;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;
use itertools::Either;

use crate::color_palette::{ColorPalette, PaletteDormant, sleep, wake, watch_palette};
use crate::color_push_button::{ColorPushButton, ColorSquare, ColorSquareRef};
use crate::float::{Float, UiViewport, fit_vertically, keep_in_window_x};
use crate::focus::{AccessibilitySync, popup_focus_lost, popup_key_closes};
use crate::list::{ItemView, ListLayout};
use crate::popup::{self, ButtonLook, PopupPointers};
use crate::push_button::NoDefaultHover;
use crate::scroll_area::ScrollArea;
use crate::select::{Item, Selection, SelectionParam, SelectionValue};
use crate::tailwind::color_name;
use crate::theme::Theme;
use crate::{MouseEvent, MouseKind, color_push_button};

/// Registers the systems and observers of [`ColorPaletteCombo`].
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
                sync_edited_item_colors,
            ),
        )
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
        .add_systems(Last, sleep_closed_popups)
        .add_systems(
            PostUpdate,
            place_opened_popups.after(bevy::ui::UiSystems::PostLayout),
        )
        .add_observer(on_insert_combo)
        .add_observer(on_remove_combo)
        .add_observer(on_insert_state)
        .add_observer(close_on_swatch_click);
    }
}

#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
/// A button that opens a [`ColorPalette`] in a popup.
///
/// Any [`Selection`] works. The button's color square previews the first selected
/// [`Item<Color>`].
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
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct ColorPaletteComboButton;

/// Marks the popup of a [`ColorPaletteCombo`].
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Hovered)]
pub struct ColorPaletteComboFloat;

/// Builds a [`ColorPaletteCombo`]: its button and its popup, both children of the combo.
///
/// The popup is a [`Float`] named `<combo name> popup`, taken at build time.
fn on_insert_combo(
    t: On<Insert<ColorPaletteCombo>>,
    mut commands: Commands,
    list_layouts: Query<&ListLayout>,
    selections: SelectionParam,
    names: Query<&Name>,
    built: Query<(), With<SubRefs>>,
    theme: Res<Theme>,
) {
    let combo = t.event().event_target();
    if built.contains(combo) {
        return;
    }
    let popup_name = popup_name(names.get(combo).ok());

    let mut button_id = Entity::PLACEHOLDER;
    let mut float_id = Entity::PLACEHOLDER;
    let palette_id = commands.spawn_empty().id();

    let list_layout = list_layouts.get(combo).ok().copied();
    let selection_ref = selections.get_ref(combo);
    commands.entity(combo).with_children(|parent| {
        button_id = parent
            .spawn((
                ColorPaletteComboButton,
                Name::new("ColorPaletteComboButton"),
                ColorPushButton,
                NoDefaultHover,
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
                BorderColor::all(theme.colors().border_highlight),
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
                            PaletteDormant,
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
    commands.entity(combo).insert((
        ColorPaletteComboState::Closed,
        sub_refs,
        ColorPaletteComboRefs {
            button: button_id,
            popup: float_id,
            palette: palette_id,
        },
    ));

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
    states: Query<&ColorPaletteComboState>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let Ok(sub_refs) = sub_refs.get(input.event_target()) else {
        return;
    };
    if states.get(sub_refs.parent) == Ok(&ColorPaletteComboState::Open)
        && popup_key_closes(&input.event().input, selections.get(sub_refs.parent))
    {
        commands
            .entity(sub_refs.parent)
            .insert(ColorPaletteComboState::Closed);
    }
}

/// Closes the combo when the focus leaves its palette.
fn close_on_focus_lost(
    lost: On<FocusLost>,
    sub_refs: Query<&SubRefs>,
    states: Query<&ColorPaletteComboState>,
    pointers: PopupPointers,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let Ok(sub_refs) = sub_refs.get(lost.event().entity) else {
        return;
    };
    let Ok(&state) = states.get(sub_refs.parent) else {
        return;
    };
    if sub_refs.palette != lost.event().entity {
        return;
    }
    if state == ColorPaletteComboState::Open && pointers.pressed_over(sub_refs.button) {
        return;
    }
    let popup_hovered = pointers.over(None, sub_refs.float_id);
    if popup_focus_lost(
        &mut focus,
        state == ColorPaletteComboState::Open,
        sub_refs.button,
        sub_refs.palette,
        popup_hovered,
    ) {
        commands
            .entity(sub_refs.parent)
            .insert(ColorPaletteComboState::Closed);
    }
}

fn on_remove_combo(
    t: On<Remove<ColorPaletteCombo>>,
    sub_refs: Query<&SubRefs>,
    mut commands: Commands,
) {
    let combo = t.event().event_target();
    if let Ok(sub_refs) = sub_refs.get(combo) {
        commands.entity(sub_refs.palette).try_despawn();
        commands.entity(sub_refs.float_id).try_despawn();
        commands.entity(sub_refs.button).try_despawn();
        commands
            .entity(combo)
            .try_remove::<(SubRefs, ColorPaletteComboRefs, ColorPaletteComboState)>();
    }
}

/// The parts of a [`ColorPaletteCombo`], on the combo itself. Only the widget builds it.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ColorPaletteComboRefs {
    button: Entity,
    popup: Entity,
    palette: Entity,
}

impl ColorPaletteComboRefs {
    /// The [`ColorPaletteComboButton`].
    #[must_use]
    pub const fn button(&self) -> Entity {
        self.button
    }

    /// The popup, a [`ColorPaletteComboFloat`].
    #[must_use]
    pub const fn popup(&self) -> Entity {
        self.popup
    }

    /// The popup's [`ColorPalette`].
    #[must_use]
    pub const fn palette(&self) -> Entity {
        self.palette
    }
}

/// Routing record on the combo and on its button and palette.
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
pub enum ColorPaletteComboState {
    /// The popup is showing.
    Open,
    /// The popup is hidden.
    Closed,
}

impl ColorPaletteComboState {
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
    t: On<Insert<ColorPaletteComboState>>,
    state: Query<(&ColorPaletteComboState, &SubRefs)>,
    mut accessibility_nodes: Query<&mut AccessibilityNode>,
    mut focus: ResMut<InputFocus>,
    mut commands: Commands,
) {
    let Ok((&state, &sub_refs)) = state.get(t.event().event_target()) else {
        return;
    };
    let open = state == ColorPaletteComboState::Open;
    popup::show_popup(&mut commands, open, sub_refs.palette, sub_refs.float_id);
    popup::move_focus(&mut focus, open, sub_refs.palette, sub_refs.button);
    if let Ok(mut node) = accessibility_nodes.get_mut(sub_refs.button) {
        popup::set_expanded(&mut node, open);
    }
    if open {
        // Hide the popup until `place_opened_popups` has placed it.
        commands.run_system_cached_with(wake, sub_refs.palette);
        commands
            .entity(sub_refs.float_id)
            .remove::<PendingSleep>()
            .insert((Visibility::Hidden, PendingPlacement(sub_refs)));
    } else {
        // Sleep at the end of the frame: the `ListBox` is still handling the pick that closed it.
        commands
            .entity(sub_refs.float_id)
            .remove::<PendingPlacement>()
            .insert(PendingSleep(sub_refs.palette));
    }
}

#[derive(Component, Debug, Copy, Clone)]
struct PendingSleep(Entity);

fn sleep_closed_popups(pending: Query<(Entity, &PendingSleep)>, mut commands: Commands) {
    for (float, &PendingSleep(palette)) in &pending {
        commands.run_system_cached_with(sleep, palette);
        commands.entity(float).remove::<PendingSleep>();
    }
}

#[derive(Component, Debug, Copy, Clone)]
struct PendingPlacement(SubRefs);

/// Places the popups that just opened once layout has sized them, then shows them.
fn place_opened_popups(pending: Query<(Entity, &PendingPlacement)>, mut commands: Commands) {
    for (float, &PendingPlacement(sub_refs)) in &pending {
        commands.run_system_cached_with(position_float, sub_refs);
        commands
            .entity(float)
            .remove::<PendingPlacement>()
            .insert(Visibility::Visible);
    }
}

/// Places the popup of a combo under its button, centered on it, in viewport coordinates.
fn position_float(
    sub_refs: In<SubRefs>,
    viewport: UiViewport,
    mut dims: Query<(&mut Node, &ComputedNode), With<ColorPaletteComboFloat>>,
    parent_dims: Query<
        (&ComputedNode, &UiGlobalTransform),
        (
            With<ColorPaletteComboButton>,
            Without<ColorPaletteComboFloat>,
        ),
    >,
) {
    const BUTTON_FLOAT_DIST: f32 = 3.0;

    let (Ok((mut node, float_computed_node)), Ok((button_computed_node, button_transform))) = (
        dims.get_mut(sub_refs.float_id),
        parent_dims.get(sub_refs.button),
    ) else {
        return;
    };

    let Some(target) = viewport.size_of(sub_refs.button) else {
        return;
    };
    let button_size = button_computed_node.size() * button_computed_node.inverse_scale_factor();
    let float_size = float_computed_node.size() * float_computed_node.inverse_scale_factor();
    let button_center = button_transform.translation * button_computed_node.inverse_scale_factor();

    let centered_x = button_center.x - float_size.x / 2.0;
    node.left = Val::Px(keep_in_window_x(centered_x, float_size.x, target.x));
    let fit = fit_vertically(
        button_center.y - button_size.y / 2.0,
        button_center.y + button_size.y / 2.0,
        BUTTON_FLOAT_DIST,
        float_size.y,
        target.y,
    );
    node.top = Val::Px(fit.top);
    node.max_height = Val::Px(fit.max_height);
    node.overflow.y = OverflowAxis::Scroll;
}

/// The colors of the selected items that have an [`Item<Color>`], in selection order.
fn selected_colors<'a>(
    selection: &'a SelectionValue,
    items: &'a Query<&Item<Color>>,
) -> impl Iterator<Item = Color> + 'a {
    selection
        .iter()
        .filter_map(|selected| items.get(selected).ok())
        .map(|item| item.data)
}

/// Keeps the popup's palette and the button's color square in step with the combo's selection.
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
            if selection_ref != changed_selection_ref
                && color_palette_combo_ref != changed_selection_ref
            {
                continue;
            }

            if selections
                .get_ref(sub_refs.palette)
                .is_none_or(|existing_ref| existing_ref != selection_ref)
            {
                commands
                    .entity(sub_refs.palette)
                    .insert(Selection::reference(selection_ref));
            } else {
                commands.run_system_cached_with(watch_palette, sub_refs.palette);
            }

            let Ok(color_square_ref) = color_square_refs.get(sub_refs.button) else {
                continue;
            };
            let Ok(mut color_square_background) = color_squares.get_mut(color_square_ref.entity())
            else {
                continue;
            };

            let color = selected_colors(selection, &items)
                .next()
                .unwrap_or(Color::Srgba(Srgba::NONE));
            color_square_background.0 = color;
            commands.run_system_cached_with(color_push_button::update_theme, Some(sub_refs.button));
        }
    }
}

/// Shows the edited color of a selected [`Item<Color>`] in the button's square.
fn sync_edited_item_colors(
    changed: Query<(), Changed<Item<Color>>>,
    combos: Query<(Entity, &SubRefs), With<ColorPaletteCombo>>,
    color_square_refs: Query<&ColorSquareRef>,
    mut color_squares: Query<&mut BackgroundColor, With<ColorSquare>>,
    selections: SelectionParam,
    items: Query<&Item<Color>>,
    mut commands: Commands,
) {
    if changed.is_empty() {
        return;
    }
    for (combo, sub_refs) in &combos {
        let Some(selection) = selections.get(combo) else {
            continue;
        };
        let Ok(color_square_ref) = color_square_refs.get(sub_refs.button) else {
            continue;
        };
        let Ok(mut square) = color_squares.get_mut(color_square_ref.entity()) else {
            continue;
        };
        let color = selected_colors(selection, &items)
            .next()
            .unwrap_or(Color::Srgba(Srgba::NONE));
        if square.0 != color {
            square.0 = color;
            commands.run_system_cached_with(color_push_button::update_theme, Some(sub_refs.button));
        }
    }
}

/// Keeps the button of each combo a combo box. Run after Bevy's button sync.
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
        let value = selections.get(combo).map_or_else(String::new, |selection| {
            selected_colors(selection, &items)
                .map(color_name)
                .collect::<Vec<_>>()
                .join(", ")
        });
        popup::sync_combo_box(&mut node, &value, sub_refs.palette);
    }
}

/// Closes an open combo when a pointer lifts its primary button, or a finger, outside of it.
///
/// The button toggles on [`Activate`] and a swatch pick closes through [`close_on_swatch_click`].
fn close_combos_on_outside_click(
    mut inputs: MessageReader<PointerInput>,
    pointers: PopupPointers,
    combos: Query<(Entity, &ColorPaletteComboState, &SubRefs), With<ColorPaletteCombo>>,
    mut combo_buttons: Query<
        (&mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for pointer in popup::primary_releases(&mut inputs) {
        if pointers.panned(pointer) {
            continue;
        }
        for (combo_id, &combo_state, subs) in combos.iter() {
            let Ok((mut background_color, mut border_color)) = combo_buttons.get_mut(subs.button)
            else {
                continue;
            };
            if popup::closes_on_outside_release(
                combo_state == ColorPaletteComboState::Open,
                pointers.over(Some(pointer), subs.button),
                pointers.over(Some(pointer), subs.float_id),
            ) {
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

/// Closes the popup of a combo and puts its button back to the resting colors.
fn close(
    combo: Entity,
    background_color: &mut BackgroundColor,
    border_color: &mut BorderColor,
    theme: &Theme,
    commands: &mut Commands,
) {
    popup::apply_look(ButtonLook::Resting, theme, background_color, border_color);
    commands
        .entity(combo)
        .insert(ColorPaletteComboState::Closed);
}

/// Closes the popup of a combo with a selection of one item at most when a swatch is clicked.
///
/// Act on `Click`, not `Released`: a release also ends a press that began elsewhere, such as a
/// scroll thumb drag.
fn close_on_swatch_click(
    t: On<MouseEvent>,
    palettes: Query<&SubRefs, With<ColorPalette>>,
    swatches: Query<&ItemView>,
    states: Query<&ColorPaletteComboState, With<ColorPaletteCombo>>,
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
    if selection.allow_multiple() || states.get(subs.parent) != Ok(&ColorPaletteComboState::Open) {
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
    combos: Query<&ColorPaletteComboState, With<ColorPaletteCombo>>,
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

    if let Some(look) = popup::button_look(
        trigger.event().kind,
        trigger.event().hovered(),
        *combo_state == ColorPaletteComboState::Open,
    ) {
        popup::apply_look(look, &theme, &mut background_color, &mut border_color);
    }
}

fn toggle_on_activate(
    activate: On<Activate>,
    buttons: Query<&SubRefs, With<ColorPaletteComboButton>>,
    states: Query<&ColorPaletteComboState, With<ColorPaletteCombo>>,
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
        *border_color = BorderColor::all(theme.colors().border_highlight);
    }
}
