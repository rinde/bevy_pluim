//! `IconButton`: a borderless button showing only an icon, with an accessible name.

use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Button;
use itertools::Either;

use crate::focus::TabStop;
use crate::icon::{Icon, IconColor};
use crate::theme::{Theme, ThemeColors};
use crate::{MouseEvent, MouseKind};

/// Registers the systems and observers of [`IconButton`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct IconButtonPlugin;

impl Plugin for IconButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_icon_buttons)
                    .run_if(resource_changed::<Theme>),
                restyle_on_disabled_change,
                sync_accessible_names,
            ),
        );
    }
}

/// The two sizes of an [`IconButton`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum IconButtonSize {
    /// A 44 px button with a 22 px icon.
    #[default]
    Regular,
    /// A 30 px button with a 16 px icon, whose icon is secondary text until the
    /// pointer is on the button.
    Small,
}

impl IconButtonSize {
    /// The side of the square button, in logical pixels.
    #[must_use]
    pub const fn button_px(self) -> f32 {
        match self {
            Self::Regular => 44.0,
            Self::Small => 30.0,
        }
    }

    /// The side of the square icon, in logical pixels.
    #[must_use]
    pub const fn icon_px(self) -> f32 {
        match self {
            Self::Regular => 22.0,
            Self::Small => 16.0,
        }
    }
}

/// A button that shows an [`Icon`] and nothing else: no border, the control
/// radius and the primary background, the hover background under the pointer.
///
/// It is a [`bevy::ui_widgets::Button`], so an action observes
/// [`Activate`](bevy::ui_widgets::Activate) on it, and it is disabled by
/// inserting [`InteractionDisabled`]. `name` is its accessible name, the label
/// of its [`AccessibilityNode`], since the icon has no text.
///
/// Insert an `IconButton` again to change the icon (play and pause) or the name
/// of a live button.
#[derive(Component, Clone, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    BackgroundColor::default(),
    Name::new("IconButton"),
    Button,
    Hovered,
    TabStop,
)]
pub struct IconButton {
    /// The icon image, white on transparent (see [`Icon`]).
    pub icon: Handle<Image>,
    /// The accessible name of the button.
    pub name: String,
    /// The size of the button.
    pub size: IconButtonSize,
}

/// The [`Icon`] child of an [`IconButton`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct IconButtonRefs {
    /// The icon.
    pub icon_id: Entity,
}

impl IconButton {
    /// A button of `size` showing `icon`, named `name` for assistive tech.
    pub fn new(icon: Handle<Image>, name: impl Into<String>, size: IconButtonSize) -> Self {
        Self {
            icon,
            name: name.into(),
            size,
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        if world.get::<IconButtonRefs>(entity).is_none() {
            let mut commands = world.commands();
            let icon_id = commands
                .spawn((
                    Icon::bundle(Handle::default(), 0.0),
                    Pickable::IGNORE,
                    ChildOf(entity),
                ))
                .id();
            commands
                .entity(entity)
                .insert(IconButtonRefs { icon_id })
                .observe(handle_hover);
        }
        world
            .commands()
            .run_system_cached_with(theme_icon_buttons, Some(entity));
    }
}

/// The background and the icon colour of a button, from its state.
const fn look(
    colors: &ThemeColors,
    size: IconButtonSize,
    disabled: bool,
    pressed: bool,
    hovered: bool,
) -> (Color, IconColor) {
    if disabled {
        return (colors.background_primary, IconColor::TextDisabled);
    }
    let background = if pressed {
        colors.selected_hovered_item
    } else if hovered {
        colors.hovered_item
    } else {
        colors.background_primary
    };
    let icon = match size {
        IconButtonSize::Small if !hovered => IconColor::TextSecondary,
        _ => IconColor::TextPrimary,
    };
    (background, icon)
}

fn handle_hover(
    trigger: On<MouseEvent>,
    mut buttons: Query<
        (&IconButton, &IconButtonRefs, &mut BackgroundColor),
        Without<InteractionDisabled>,
    >,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let kind = trigger.event().kind;
    if matches!(kind, MouseKind::Click(_) | MouseKind::Drag(_)) {
        return;
    }
    let pressed = matches!(kind, MouseKind::Pressed);
    let hovered = !matches!(kind, MouseKind::Out);
    let target = trigger.event().event_target();
    if let Ok((button, refs, mut background)) = buttons.get_mut(target) {
        let (color, icon) = look(&theme.colors, button.size, false, pressed, hovered);
        *background = BackgroundColor(color);
        commands.entity(refs.icon_id).insert(Icon { color: icon });
    }
}

/// Restyles an [`IconButton`] whose [`InteractionDisabled`] was just added or removed.
fn restyle_on_disabled_change(
    added: Query<Entity, (With<IconButton>, Added<InteractionDisabled>)>,
    mut removed: RemovedComponents<InteractionDisabled>,
    buttons: Query<(), With<IconButton>>,
    mut commands: Commands,
) {
    for entity in added
        .iter()
        .chain(removed.read().filter(|e| buttons.contains(*e)))
    {
        commands.run_system_cached_with(theme_icon_buttons, Some(entity));
    }
}

/// Keeps the label of each button's [`AccessibilityNode`] at its name. Bevy
/// clears the label of a button without text when the button is added, so this
/// also sets it again after that.
fn sync_accessible_names(
    buttons: Query<(Entity, &IconButton), With<AccessibilityNode>>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    for (entity, button) in &buttons {
        let Ok(mut node) = nodes.get_mut(entity) else {
            continue;
        };
        if node.label() != Some(button.name.as_str()) {
            node.set_label(button.name.as_str());
        }
    }
}

fn theme_icon_buttons(
    button_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut buttons: Query<(
        &IconButton,
        &IconButtonRefs,
        &mut BackgroundColor,
        &mut Node,
        Has<InteractionDisabled>,
        Has<Pressed>,
        &Hovered,
    )>,
    icons: Query<(&Icon, &ImageNode, &Node), Without<IconButton>>,
    mut commands: Commands,
) {
    let it = if let Some(button_id) = button_id.as_ref() {
        Either::Left(buttons.get_mut(*button_id).into_iter())
    } else {
        Either::Right(buttons.iter_mut())
    };
    for (button, refs, mut background, mut node, disabled, pressed, hovered) in it {
        let (color, icon_color) =
            look(&theme.colors, button.size, disabled, pressed, hovered.get());
        background.0 = color;
        node.width = Val::Px(button.size.button_px());
        node.height = Val::Px(button.size.button_px());
        node.border_radius = BorderRadius::all(theme.control_radius);

        let Ok((icon, image, icon_node)) = icons.get(refs.icon_id) else {
            continue;
        };
        let side = Val::Px(button.size.icon_px());
        if image.image != button.icon || icon_node.width != side || icon_node.height != side {
            commands
                .entity(refs.icon_id)
                .insert(Icon::bundle_with_color(
                    button.icon.clone(),
                    button.size.icon_px(),
                    icon_color,
                ));
        } else if icon.color != icon_color {
            commands
                .entity(refs.icon_id)
                .insert(Icon { color: icon_color });
        }
    }
}
