//! `PushButton`: a themed button with a label and an optional icon.
//!
//! This is the reference widget for how widgets in this crate are put together.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Button;
use itertools::Either;

use bevy::a11y::AccessibilityNode;

use crate::combo::ComboButton;
use crate::focus::{AccessibilitySync, TabStop, sync_label};
use crate::theme::{Theme, ThemeColors};
use crate::{MouseEvent, MouseKind};

/// Registers the systems and observers of [`PushButton`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct PushButtonPlugin;

impl Plugin for PushButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_push_button)
                    .run_if(resource_changed::<Theme>),
                restyle_on_disabled_change,
                restyle_on_size_change,
            ),
        )
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync));
    }
}

/// Keeps the label of each button's [`AccessibilityNode`] at its text, without the icon. Bevy
/// leaves a [`bevy::ui_widgets::Button`] unlabelled. A combo's button is left alone: it is a
/// combo box whose value says the selection.
fn sync_accessibility(
    mut buttons: Query<(&PushButton, &mut AccessibilityNode), Without<ComboButton>>,
) {
    for (button, mut node) in &mut buttons {
        sync_label(&mut node, &button.label);
    }
}

/// A themed button with a label and an optional icon.
///
/// It is a [`bevy::ui_widgets::Button`]: it is [`Pressed`] while
/// held, triggers [`Activate`](bevy::ui_widgets::Activate) when it is used (a
/// click, or Enter or Space on a focused button) and is disabled by inserting
/// [`InteractionDisabled`]. React to a use of the button by observing
/// `Activate`; the hover and press colours follow the pointer on their own, and
/// stay right when the [`Theme`] changes under a pointer that rests on the button.
/// Its [`Hovered`] says whether the pointer is on it.
#[derive(Component, Clone, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
#[require(BoxShadow::default(),
    Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    },
    Name::new("PushButton"),
    ButtonSize,
    Button,
    Hovered,
    TabStop,
)]
pub struct PushButton {
    label: String,
    icon: Option<String>,
}

/// The size of a [`PushButton`]. Insert `ButtonSize::Compact` with the button, or later; a theme
/// change keeps it.
///
/// `Regular` is padded by [`Theme::item_padding`], has a 14 px label and a small shadow.
/// `Compact` is [`Theme::compact_button_height`] high with [`Theme::compact_button_padding`], a
/// smaller radius, a semibold 12 px label and no shadow, for buttons inside a row.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub enum ButtonSize {
    /// The usual size.
    #[default]
    Regular,
    /// A small button for a row of other content.
    Compact,
}

/// Marks the label text of a [`PushButton`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonLabel;

/// The children of a [`PushButton`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonRefs {
    /// The label text.
    pub label_id: Entity,
    /// The icon text, if the button has an icon.
    pub icon_id: Option<Entity>,
}

/// Marks the icon text of a [`PushButton`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonIcon;

/// Marker component that disables the default hover effect for `PushButton`s.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoDefaultHover;

impl PushButton {
    /// A button with a label and an icon.
    pub fn new(label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: Some(icon.into()),
        }
    }

    /// A button with only a label.
    pub fn from_label(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = world.entity(context.entity);
        let Some(button) = entity.get::<Self>() else {
            return;
        };
        let icon = button
            .icon
            .as_ref()
            .map(|s| (Text::new(s), PushButtonIcon, Pickable::IGNORE));
        let label = (Text::new(&button.label), PushButtonLabel, Pickable::IGNORE);

        let default_hover = !entity.contains::<NoDefaultHover>();

        let mut commands = world.commands();
        let mut entity_commands = commands.entity(context.entity);

        if default_hover {
            entity_commands.observe(handle_hover);
        }
        let mut label_id = Entity::PLACEHOLDER;
        let mut icon_id = None;
        world
            .commands()
            .entity(context.entity)
            .with_children(|spawner| {
                label_id = spawner.spawn(label).id();
                if let Some(icon) = icon {
                    icon_id = Some(spawner.spawn(icon).id());
                }
            });
        world
            .commands()
            .entity(context.entity)
            .insert(PushButtonRefs { label_id, icon_id });

        world
            .commands()
            .run_system_cached_with(theme_push_button, Some(context.entity));
    }
}

/// The background of a [`PushButton`] with the default hover, from its state.
const fn background_for(
    colors: &ThemeColors,
    disabled: bool,
    pressed: bool,
    hovered: bool,
) -> Color {
    if disabled {
        colors.background_primary
    } else if pressed {
        colors.selected_hovered_item
    } else if hovered {
        colors.hovered_item
    } else {
        colors.background_primary
    }
}

fn handle_hover(
    trigger: On<MouseEvent>,
    mut query: Query<&mut BackgroundColor, Without<InteractionDisabled>>,
    theme: Res<Theme>,
) {
    // A click and a drag say nothing new about the pointer being on the button: the over,
    // press, release and out events around them set the colours. A drag in particular can
    // arrive after the pointer left the button, so it must not put it back under the pointer.
    let kind = trigger.event().kind;
    if matches!(kind, MouseKind::Click(_) | MouseKind::Drag) {
        return;
    }
    // The pointer is on the button for every event but `Out`, a release included.
    let pressed = matches!(kind, MouseKind::Pressed);
    let hovered = !matches!(kind, MouseKind::Out);
    if let Ok(mut background) = query.get_mut(trigger.event().event_target()) {
        *background = BackgroundColor(background_for(&theme.colors, false, pressed, hovered));
    }
}

/// Restyles a [`PushButton`] whose [`InteractionDisabled`] was just added or removed.
fn restyle_on_disabled_change(
    added: Query<Entity, (With<PushButton>, Added<InteractionDisabled>)>,
    mut removed: RemovedComponents<InteractionDisabled>,
    push_buttons: Query<(), With<PushButton>>,
    mut commands: Commands,
) {
    for entity in added
        .iter()
        .chain(removed.read().filter(|e| push_buttons.contains(*e)))
    {
        commands.run_system_cached_with(theme_push_button, Some(entity));
    }
}

/// Restyles a [`PushButton`] whose [`ButtonSize`] was changed after it was spawned.
fn restyle_on_size_change(
    changed: Query<Entity, (With<PushButtonRefs>, Changed<ButtonSize>)>,
    mut commands: Commands,
) {
    for entity in &changed {
        commands.run_system_cached_with(theme_push_button, Some(entity));
    }
}

fn theme_push_button(
    button_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut buttons: Query<
        (
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BoxShadow,
            &mut Node,
            &PushButtonRefs,
            Has<InteractionDisabled>,
            Has<Pressed>,
            &Hovered,
            Has<NoDefaultHover>,
            &ButtonSize,
        ),
        With<PushButton>,
    >,
    mut labels: Query<
        (&mut TextColor, &mut TextFont),
        (With<PushButtonLabel>, Without<PushButtonIcon>),
    >,
    mut icons: Query<
        (&mut TextColor, &mut TextFont),
        (With<PushButtonIcon>, Without<PushButtonLabel>),
    >,
) {
    let it = if let Some(button_id) = button_id.as_ref() {
        Either::Left(buttons.get_mut(*button_id).into_iter())
    } else {
        Either::Right(buttons.iter_mut())
    };
    for (
        mut background_color,
        mut border_color,
        mut box_shadow,
        mut node,
        refs,
        disabled,
        pressed,
        hovered,
        no_default_hover,
        size,
    ) in it
    {
        // A button with `NoDefaultHover` is coloured by its owner, so it rests here.
        background_color.0 = background_for(
            &theme.colors,
            disabled,
            pressed && !no_default_hover,
            hovered.get() && !no_default_hover,
        );
        border_color.set_all(theme.colors.border);
        node.border = UiRect::all(theme.border_thickness);
        let compact = *size == ButtonSize::Compact;
        if compact {
            node.border_radius = BorderRadius::all(theme.compact_button_radius);
            *box_shadow = BoxShadow::default();
            node.padding = theme.compact_button_padding;
            node.height = theme.compact_button_height;
        } else {
            node.border_radius = BorderRadius::all(theme.control_radius);
            *box_shadow = theme.small_shadow().into();
            node.padding = theme.item_padding;
            node.height = Val::Auto;
        }
        let (font_size, weight) = if compact {
            (theme.font_size_small, FontWeight::SEMIBOLD)
        } else {
            (theme.font_size_secondary, FontWeight::default())
        };

        // label
        if let Ok((mut text_color, mut text_font)) = labels.get_mut(refs.label_id) {
            *text_color = TextColor(if disabled {
                theme.colors.text_disabled
            } else {
                theme.colors.text_primary
            });
            *text_font = TextFont {
                font: theme.body_font.clone().into(),
                font_size: FontSize::Px(font_size),
                weight,
                ..default()
            };
        }

        // icon
        if let Some(icon_id) = refs.icon_id
            && let Ok((mut text_color, mut text_font)) = icons.get_mut(icon_id)
        {
            *text_color = TextColor(if disabled {
                theme.colors.text_disabled
            } else {
                theme.colors.text_primary
            });
            *text_font = TextFont {
                font: theme.body_font.clone().into(),
                font_size: FontSize::Px(font_size),
                weight,
                ..default()
            };
        }
    }
}
