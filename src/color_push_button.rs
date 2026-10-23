//! `ColorPushButton`: a button showing a colour square.

use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Button;
use itertools::Either;

use crate::color_palette_combo::ColorPaletteComboButton;
use crate::focus::TabStop;
use crate::tailwind::color_name;
use crate::theme::Theme;

/// Registers the systems and observers of [`ColorPushButton`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct ColorPushButtonPlugin;

impl Plugin for ColorPushButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(update_theme)
                    .run_if(resource_changed::<Theme>),
                restyle_on_disabled_change,
                sync_accessible_names,
            ),
        );
    }
}

/// A button that shows a colour square instead of a label.
///
/// Like [`PushButton`](crate::push_button::PushButton) it is a
/// [`bevy::ui_widgets::Button`] and triggers [`Activate`](bevy::ui_widgets::Activate).
/// Its accessible name is the name of the colour it shows: a Tailwind name
/// (`blue-500`) or else a hex (`#3b82f6`).
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
#[require(BoxShadow::default(),
    Node{
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    },
    Name::new("ColorPushButton"),Button,TabStop,
)]
pub struct ColorPushButton;

/// Marks the colour square of a [`ColorPushButton`].
#[derive(Component, Clone, Copy, Debug, Reflect)]
#[reflect(Component)]
pub struct ColorSquare;

/// The colour square of a [`ColorPushButton`].
#[derive(Component, Clone, Copy, Debug)]
pub struct ColorSquareRef(pub Entity);

const SQUARE_SIZE: f32 = 25.0;

impl ColorPushButton {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let mut commands = world.commands();
        let mut entity_commands = commands.entity(context.entity);

        let mut square_id = Entity::PLACEHOLDER;
        entity_commands.with_children(|parent| {
            square_id = parent
                .spawn((
                    Node {
                        width: Val::Px(SQUARE_SIZE),
                        height: Val::Px(SQUARE_SIZE),
                        ..default()
                    },
                    BackgroundColor::default(),
                    ColorSquare,
                    Pickable {
                        should_block_lower: false,
                        ..default()
                    },
                ))
                .id();
        });
        entity_commands.insert(ColorSquareRef(square_id));

        world
            .commands()
            .run_system_cached_with(update_theme, Some(context.entity));
    }
}

/// Keeps the label of each button's [`AccessibilityNode`] at the name of its colour.
/// Bevy clears the label of a button without text when it is added, so this also
/// sets it again after that. The button of a palette combo is left alone: it is a combo
/// box whose value says the colour.
fn sync_accessible_names(
    buttons: Query<
        (Entity, &ColorSquareRef),
        (With<ColorPushButton>, Without<ColorPaletteComboButton>),
    >,
    squares: Query<&BackgroundColor, With<ColorSquare>>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    for (entity, ColorSquareRef(square)) in &buttons {
        let (Ok(color), Ok(mut node)) = (squares.get(*square), nodes.get_mut(entity)) else {
            continue;
        };
        let name = color_name(color.0);
        if node.label() != Some(name.as_str()) {
            node.set_label(name);
        }
    }
}

/// Restyles a [`ColorPushButton`] whose [`InteractionDisabled`] was just added or removed.
fn restyle_on_disabled_change(
    added: Query<Entity, (With<ColorPushButton>, Added<InteractionDisabled>)>,
    mut removed: RemovedComponents<InteractionDisabled>,
    buttons: Query<(), With<ColorPushButton>>,
    mut commands: Commands,
) {
    for entity in added
        .iter()
        .chain(removed.read().filter(|e| buttons.contains(*e)))
    {
        commands.run_system_cached_with(update_theme, Some(entity));
    }
}

pub(crate) fn update_theme(
    button_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut buttons: Query<
        (
            &ColorSquareRef,
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BoxShadow,
            &mut Node,
            Has<InteractionDisabled>,
        ),
        With<ColorPushButton>,
    >,
    mut squares: Query<
        (&mut Node, &mut BorderColor, &BackgroundColor),
        (With<ColorSquare>, Without<ColorPushButton>),
    >,
) {
    let it = if let Some(button_id) = button_id.as_ref() {
        Either::Left(buttons.get_mut(*button_id).into_iter())
    } else {
        Either::Right(buttons.iter_mut())
    };
    for (
        ColorSquareRef(square_ref),
        mut background_color,
        mut border_color,
        mut box_shadow,
        mut node,
        disabled,
    ) in it
    {
        background_color.0 = theme.colors.background_primary;
        border_color.set_all(if disabled {
            theme.colors.text_disabled
        } else {
            theme.colors.border
        });
        node.border_radius = BorderRadius::all(theme.control_radius);
        *box_shadow = theme.small_shadow().into();
        node.border = UiRect::all(theme.border_thickness);
        node.padding = theme.color_button_padding;

        let Ok((mut square_node, mut square_border_color, square_color)) =
            squares.get_mut(*square_ref)
        else {
            continue;
        };

        if square_color.0.luminance() < 0.5 {
            square_border_color.set_all(square_color.0.lighter(0.1));
        } else {
            square_border_color.set_all(square_color.0.darker(0.1));
        }

        square_node.border = UiRect::all(Val::Px(1.0));
        square_node.border_radius = BorderRadius::all(Val::Px(5.0));
    }
}
