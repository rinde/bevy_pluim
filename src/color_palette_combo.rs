//! `ColorPaletteCombo`: a [`ColorPalette`] inside a combo-style popup.

use bevy::a11y::AccessibilityNode;
use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
use bevy::ui_widgets::Activate;
use bevy::window::PrimaryWindow;
use itertools::Either;

use crate::color_palette::ColorPalette;
use crate::color_push_button::{ColorPushButton, ColorSquare, ColorSquareRef};
use crate::float::Float;
use crate::list::ListLayout;
use crate::push_button::NoDefaultHover;
use crate::scroll_area::ScrollArea;
use crate::select::{Item, Selection, SelectionParam};
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
        .add_observer(on_insert_combo)
        .add_observer(on_remove_combo)
        .add_observer(on_insert_state);
    }
}

#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
// #[component(on_add=Self::on_insert)]
/// A button that opens a [`ColorPalette`] in a popup.
#[require(
    Node {
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Name::new("ColorPaletteCombo"),
    RelativeCursorPosition::default()
)]
pub struct ColorPaletteCombo;

#[derive(Component, Debug, Copy, Clone)]
struct ColorPaletteComboButton;

/// Marks the popup of a [`ColorPaletteCombo`].
#[derive(Component, Debug, Copy, Clone)]
struct ColorPaletteComboFloat;

/// Builds a [`ColorPaletteCombo`]: its button, as a child, and its popup.
///
/// The popup is a child of the combo's own topmost ancestor, not of the combo,
/// so that nothing between them (a scroll area, a card) clips it or draws over
/// it. It is not looked up by [`UiRoot`](crate::UiRoot): an app may have more
/// than one, so the combo's own root is the one that is certain to be on its
/// screen. The parent must be there when the combo is inserted, so spawn the
/// combo as a child. A combo without a parent has no such root, so it logs an
/// `error!` and gets neither button nor popup.
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
    t: On<Insert, ColorPaletteCombo>,
    mut commands: Commands,
    list_layouts: Query<&ListLayout>,
    selections: SelectionParam,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
) {
    let combo = t.event().event_target();
    if !parents.contains(combo) {
        error!(
            "ColorPaletteCombo {combo} has no parent, so it has no UI root to hold its popup: \
             spawn it as a child"
        );
        return;
    }
    let root = parents.root_ancestor(combo);
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
    });
    commands.entity(root).with_children(|parent| {
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
                RelativeCursorPosition::default(),
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

fn on_remove_combo(
    t: On<Remove, ColorPaletteCombo>,
    sub_refs: Query<&SubRefs>,
    mut commands: Commands,
) {
    if let Ok(sub_refs) = sub_refs.get(t.event().event_target()) {
        commands.entity(sub_refs.float_id).despawn();
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
    t: On<Insert, State>,
    state: Query<(&State, &SubRefs)>,
    mut accessibility_nodes: Query<&mut AccessibilityNode>,
    mut commands: Commands,
) {
    let Ok((&state, &sub_refs)) = state.get(t.event().event_target()) else {
        return;
    };

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

/// Places the popup of a combo under its button, centred on it.
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

/// System that closes the combo when clicking outside of it.
fn close_combos_on_outside_click(
    mouse_button: Res<ButtonInput<MouseButton>>,
    combos: Query<(Entity, &RelativeCursorPosition, &State, &SubRefs), With<ColorPaletteCombo>>,
    mut combo_buttons: Query<
        (&mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    selections: SelectionParam,
    floats: Query<&RelativeCursorPosition, With<ColorPaletteComboFloat>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    if mouse_button.just_released(MouseButton::Left) {
        for (combo_id, rcp, &combo_state, subs) in combos.iter() {
            let Some(selection) = selections.get(combo_id) else {
                continue;
            };
            let over_float = floats
                .get(subs.float_id)
                .is_ok_and(RelativeCursorPosition::cursor_over);
            if combo_state == State::Open
                && !rcp.cursor_over()
                && !(selection.allow_multiple() && over_float)
            {
                let Ok((mut background_color, mut border_color)) =
                    combo_buttons.get_mut(subs.button)
                else {
                    continue;
                };

                *background_color = BackgroundColor(theme.colors.background_primary);
                *border_color = BorderColor::all(theme.colors.border);
                commands.entity(combo_id).insert(State::Closed);
            }
        }
    }
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
        (MouseKind::Over | MouseKind::Pressed, _) | (_, State::Open) => {
            *background_color = BackgroundColor(theme.colors.background_secondary);
            *border_color = BorderColor::all(theme.colors.border_highlight);
        }

        (MouseKind::Out, State::Closed) => {
            *background_color = BackgroundColor(theme.colors.background_primary);
            *border_color = BorderColor::all(theme.colors.border);
        }
        (MouseKind::Released, State::Closed) => {}
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
