use bevy::prelude::*;
use bevy::ui::RelativeCursorPosition;
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
use crate::{MouseEvent, UiRoot, color_push_button};

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

#[derive(Component, Debug, Copy, Clone, Default)]
// #[component(on_add=Self::on_insert)]
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

#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("ColorPaletteComboFloat"))]
struct ColorPaletteComboFloat;

fn on_insert_combo(
    t: Trigger<OnInsert, ColorPaletteCombo>,
    mut commands: Commands,
    list_layouts: Query<&ListLayout>,
    root: Query<Entity, With<UiRoot>>,
) {
    let mut button_id = Entity::PLACEHOLDER;
    let mut float_id = Entity::PLACEHOLDER;

    let list_layout = list_layouts.get(t.target()).ok().copied();
    commands.entity(t.target()).with_children(|parent| {
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
    let root = root.single().unwrap();
    commands.entity(root).with_children(|parent| {
        float_id = parent
            .spawn((
                Float,
                ColorPaletteComboFloat,
                Node {
                    left: Val::Px(0.0),
                    top: Val::Px(40.0),
                    position_type: PositionType::Absolute,
                    height: Val::Px(250.0),
                    border: UiRect::all(Val::Px(1.0)),
                    ..default()
                },
                BorderColor(Color::BLACK),
                Visibility::Hidden,
                ScrollArea::bundle(
                    move |parent| {
                        parent.spawn((
                            Pickable {
                                should_block_lower: false,
                                ..default()
                            },
                            ColorPalette,
                            list_layout.unwrap_or(ListLayout::VerticalGrid { columns: 6 }),
                        ));
                    },
                    true,
                    false,
                ),
                RelativeCursorPosition::default(),
            ))
            .id();
    });

    let sub_refs = SubRefs {
        parent: t.target(),
        button: button_id,
        float_id,
    };
    commands
        .entity(t.target())
        .insert((State::Closed, sub_refs));

    commands
        .entity(button_id)
        .insert(sub_refs)
        .observe(handle_combo_button_mouse_events);
}

fn on_remove_combo(
    t: Trigger<OnRemove, ColorPaletteCombo>,
    sub_refs: Query<&SubRefs>,
    mut commands: Commands,
) {
    let sub_refs = sub_refs.get(t.target()).unwrap();
    commands.entity(sub_refs.float_id).despawn();
}

#[derive(Component, Debug, Copy, Clone)]
struct SubRefs {
    parent: Entity,
    button: Entity,
    float_id: Entity,
}

#[derive(Component, Debug, Copy, Clone, Eq, PartialEq)]
#[component(immutable)]
pub enum State {
    Open,
    Closed,
}

impl State {
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            State::Open => State::Closed,
            State::Closed => State::Open,
        }
    }
}

fn on_insert_state(
    t: Trigger<OnInsert, State>,
    state: Query<(&State, &SubRefs)>,
    mut commands: Commands,
) {
    let (&state, &sub_refs) = state.get(t.target()).unwrap();

    let visibility = match state {
        State::Open => Visibility::Visible,
        State::Closed => Visibility::Hidden,
    };
    commands.entity(sub_refs.float_id).insert(visibility);
    if state == State::Open {
        commands.run_system_cached_with(position_float, sub_refs);
    }
}

fn position_float(
    sub_refs: In<SubRefs>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut dims: Query<(&mut Node, &GlobalTransform, &ComputedNode), With<ColorPaletteComboFloat>>,
    parent_dims: Query<
        (&ComputedNode, &GlobalTransform),
        (
            With<ColorPaletteComboButton>,
            Without<ColorPaletteComboFloat>,
        ),
    >,
) {
    const MARGIN: f32 = 10.0;
    const BUTTON_FLOAT_DIST: f32 = 3.0;

    let (mut node, global_transform, float_computed_node) =
        dims.get_mut(sub_refs.float_id).unwrap();
    let (button_computed_node, button_transform) = parent_dims.get(sub_refs.button).unwrap();

    let button_size = button_computed_node.size() * button_computed_node.inverse_scale_factor();
    let float_size =
        float_computed_node.content_size() * float_computed_node.inverse_scale_factor();
    let translation = Vec2::new(
        global_transform.translation().x,
        global_transform.translation().y,
    ) * float_computed_node.inverse_scale_factor();
    let button_translation = Vec2::new(
        button_transform.translation().x,
        button_transform.translation().y,
    ) * button_computed_node.inverse_scale_factor();

    let min_coord = translation - (float_size / 2.0);
    let max_coord = translation + (float_size / 2.0);

    let mut intended_x = button_translation.x + (-float_size.x / 2.0) + (button_size.x / 2.0);
    let mut intended_y = button_translation.y + (button_size.y / 2.0) + BUTTON_FLOAT_DIST;

    // set x position
    let Val::Px(current_x) = node.left else {
        panic!("Unexpected value type for node.left")
    };

    let delta_x = intended_x - current_x;
    let global_max_x = max_coord.x + delta_x;
    let global_min_x = min_coord.x + delta_x;

    if global_max_x + MARGIN > window.width() {
        let excess_x = global_max_x + MARGIN - window.width();
        intended_x -= excess_x;
    } else if global_min_x < MARGIN {
        let excess_x = MARGIN - global_min_x;
        intended_x += excess_x;
    }
    node.left = Val::Px(intended_x);

    // set y position
    let Val::Px(current_y) = node.top else {
        panic!("Unexpected value type for node.top")
    };

    let delta_y = intended_y - current_y;
    let global_max_y = max_coord.y + delta_y;

    if global_max_y + MARGIN > window.height() {
        intended_y =
            button_translation.y - (button_size.y / 2.0) - BUTTON_FLOAT_DIST - float_size.y;
    }
    node.top = Val::Px(intended_y);
}

fn on_selection_change(
    q: Query<Entity, Changed<Selection>>,
    combos: Query<(Entity, &SubRefs), With<ColorPaletteCombo>>,
    color_square_refs: Query<&ColorSquareRef>,
    mut color_squares: Query<&mut BackgroundColor, With<ColorSquare>>,
    selections: SelectionParam,
    items: Query<&Item<Color>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (color_palette_combo_ref, sub_refs) in combos {
            let Some((selection_ref, selection)) = selections.get_with_ref(color_palette_combo_ref)
            else {
                continue;
            };
            if selection_ref != changed_selection_ref {
                continue;
            }

            // TODO make this less brittle
            let scroll_contents_ref = children
                .get(sub_refs.float_id)
                .unwrap()
                .iter()
                .next()
                .unwrap();
            let color_palette_ref = children
                .get(scroll_contents_ref)
                .unwrap()
                .iter()
                .next()
                .unwrap();

            if selections
                .get_ref(color_palette_ref)
                .is_none_or(|existing_ref| existing_ref != changed_selection_ref)
            {
                // Selection id has changed -> propagate to the List
                commands
                    .entity(color_palette_ref)
                    .insert(Selection::reference(changed_selection_ref));
            }

            let ColorSquareRef(color_square_id) = color_square_refs.get(sub_refs.button).unwrap();
            let mut color_square_background = color_squares.get_mut(*color_square_id).unwrap();

            assert!(!selection.allow_multiple(), "Multi selection not supported");
            let color = if let Some(selected) = selection.single() {
                items.get(selected).unwrap().data
            } else {
                Color::Srgba(Srgba::NONE)
            };
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
            let selection = selections.get(combo_id).unwrap();
            if combo_state == State::Open
                && !rcp.mouse_over()
                && !(selection.allow_multiple() && floats.get(subs.float_id).unwrap().mouse_over())
            {
                let (mut background_color, mut border_color) =
                    combo_buttons.get_mut(subs.button).unwrap();

                *background_color = BackgroundColor(theme.colors.background_primary);
                *border_color = BorderColor(theme.colors.border);
                commands.entity(combo_id).insert(State::Closed);
            }
        }
    }
}

fn handle_combo_button_mouse_events(
    trigger: Trigger<MouseEvent>,
    mut combo_buttons: Query<
        (&SubRefs, &mut BackgroundColor, &mut BorderColor),
        With<ColorPaletteComboButton>,
    >,
    combos: Query<&State, With<ColorPaletteCombo>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let (
        SubRefs {
            parent: parent_id, ..
        },
        mut background_color,
        mut border_color,
    ) = combo_buttons.get_mut(trigger.target()).unwrap();
    let combo_state = combos.get(*parent_id).unwrap();

    match (trigger.event(), combo_state) {
        (MouseEvent::Pressed, _) => {
            commands.entity(*parent_id).insert(combo_state.inverse());
        }
        (MouseEvent::Over, _) | (_, State::Open) => {
            *background_color = BackgroundColor(theme.colors.background_secondary);
            *border_color = BorderColor(theme.colors.border_highlight);
        }

        (MouseEvent::Out, State::Closed) => {
            *background_color = BackgroundColor(theme.colors.background_primary);
            *border_color = BorderColor(theme.colors.border);
        }
        (MouseEvent::Released, State::Closed) => {}
    }
}

fn theme_floats(
    float_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut floats: Query<&mut BorderColor, With<ColorPaletteComboFloat>>,
) {
    let it = if let Some(float_id) = float_id.as_ref() {
        Either::Left([floats.get_mut(*float_id).unwrap()].into_iter())
    } else {
        Either::Right(floats.iter_mut())
    };

    for mut border_color in it {
        *border_color = BorderColor(theme.colors.border_highlight);
    }
}
