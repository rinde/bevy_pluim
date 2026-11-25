//! `PushButton`: a themed button with a label and an optional icon.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Button;
use itertools::Either;

use bevy::a11y::AccessibilityNode;

use crate::combo::ComboButton;
use crate::focus::{AccessibilitySync, TabStop, sync_label, sync_toggled};
use crate::icon_button::{ToggleState, toggle_on_activate};
use crate::theme::{Theme, ThemeColors};
use crate::{MouseEvent, MouseKind};

/// Registers the systems and observers of [`PushButton`].
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
                restyle_on_hover_marker_change,
            ),
        )
        .add_observer(restyle_on_toggle)
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync));
    }
}

fn restyle_on_toggle(
    t: On<Insert<ToggleState>>,
    buttons: Query<(), With<PushButton>>,
    mut commands: Commands,
) {
    let target = t.event().event_target();
    if buttons.contains(target) {
        commands.run_system_cached_with(theme_push_button, Some(target));
    }
}

/// Keeps the accessible label and toggled state in line with the text and [`ToggleState`].
/// Skips a combo's button: it is a combo box.
fn sync_accessibility(
    mut buttons: ParamSet<(
        Query<
            (&PushButton, Option<&ToggleState>, &mut AccessibilityNode),
            (
                Without<ComboButton>,
                Or<(
                    Changed<AccessibilityNode>,
                    Changed<PushButton>,
                    Changed<ToggleState>,
                )>,
            ),
        >,
        Query<(&PushButton, Option<&ToggleState>, &mut AccessibilityNode), Without<ComboButton>>,
    )>,
    mut untoggled: RemovedComponents<ToggleState>,
) {
    for (button, toggle, mut node) in &mut buttons.p0() {
        sync_label(&mut node, &button.label);
        sync_toggled(&mut node, toggle.map(|toggle| toggle.0));
    }
    let mut all = buttons.p1();
    for entity in untoggled.read() {
        if let Ok((button, toggle, mut node)) = all.get_mut(entity) {
            sync_label(&mut node, &button.label);
            sync_toggled(&mut node, toggle.map(|toggle| toggle.0));
        }
    }
}

/// A themed button with a label and an optional icon.
/// Observe [`Activate`](bevy::ui_widgets::Activate) to react; insert a [`ToggleState`] to make it
/// a toggle.
///
/// The component is immutable; insert a new value to change it:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::push_button::PushButton;
/// fn edit(mut buttons: Query<&mut PushButton>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert = Self::on_insert)]
#[require(BoxShadow::default(),
    Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    },
    Name::new("PushButton"),
    ButtonSize,
    IconPlacement,
    Button,
    Hovered,
    TabStop,
)]
pub struct PushButton {
    label: String,
    icon: Option<String>,
}

/// The size of a [`PushButton`].
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub enum ButtonSize {
    /// The usual size.
    #[default]
    Regular,
    /// A small button for a row of other content.
    Compact,
}

/// Where the icon of a [`PushButton`] sits relative to its label.
#[derive(Component, Clone, Copy, Debug, Default, Eq, PartialEq, Reflect)]
#[reflect(Component)]
pub enum IconPlacement {
    /// The icon is before the label.
    Left,
    /// The icon is after the label.
    #[default]
    Right,
    /// The icon is above the label.
    Top,
    /// The icon is below the label.
    Bottom,
}

impl IconPlacement {
    const fn flex_direction(self) -> FlexDirection {
        match self {
            Self::Left => FlexDirection::RowReverse,
            Self::Right => FlexDirection::Row,
            Self::Top => FlexDirection::ColumnReverse,
            Self::Bottom => FlexDirection::Column,
        }
    }
}

/// Marks the label text of a [`PushButton`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonLabel;

/// The children of a [`PushButton`], for reading.
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::push_button::PushButtonRefs;
/// let _ = PushButtonRefs { label_id: Entity::PLACEHOLDER, icon_id: None };
/// ```
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
#[component(immutable)]
pub struct PushButtonRefs {
    label_id: Entity,
    icon_id: Option<Entity>,
}

impl PushButtonRefs {
    /// The label text.
    #[must_use]
    pub const fn label(&self) -> Entity {
        self.label_id
    }

    /// The icon text, if the button has an icon.
    #[must_use]
    pub const fn icon(&self) -> Option<Entity> {
        self.icon_id
    }
}

/// Marks the icon text of a [`PushButton`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonIcon;

/// Disables the default hover effect of a [`PushButton`].
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

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        let Some(button) = world.get::<Self>(entity) else {
            return;
        };
        let (label, icon) = (button.label.clone(), button.icon.clone());
        let mut commands = world.commands();
        commands.queue(move |world: &mut World| {
            let Ok(mut entity_mut) = world.get_entity_mut(entity) else {
                return;
            };
            let refs = entity_mut.get::<PushButtonRefs>().copied();
            let Some(refs) = refs else {
                entity_mut.observe(toggle_on_activate);
                entity_mut.observe(handle_hover);
                let label_id = world
                    .spawn((
                        Text::new(label),
                        PushButtonLabel,
                        Pickable::IGNORE,
                        ChildOf(entity),
                    ))
                    .id();
                let icon_id = icon.map(|icon| {
                    world
                        .spawn((
                            Text::new(icon),
                            PushButtonIcon,
                            Pickable::IGNORE,
                            ChildOf(entity),
                        ))
                        .id()
                });
                world
                    .entity_mut(entity)
                    .insert(PushButtonRefs { label_id, icon_id });
                return;
            };
            set_text(world, refs.label_id, &label);
            let icon_id = match (refs.icon_id, icon) {
                (Some(icon_id), Some(icon)) => {
                    set_text(world, icon_id, &icon);
                    Some(icon_id)
                }
                (Some(icon_id), None) => {
                    world.despawn(icon_id);
                    None
                }
                (None, Some(icon)) => Some(
                    world
                        .spawn((
                            Text::new(icon),
                            PushButtonIcon,
                            Pickable::IGNORE,
                            ChildOf(entity),
                        ))
                        .id(),
                ),
                (None, None) => None,
            };
            if icon_id != refs.icon_id {
                world.entity_mut(entity).insert(PushButtonRefs {
                    label_id: refs.label_id,
                    icon_id,
                });
            }
        });
        commands.run_system_cached_with(theme_push_button, Some(entity));
    }
}

fn set_text(world: &mut World, entity: Entity, value: &str) {
    if let Some(mut text) = world.get_mut::<Text>(entity)
        && text.0 != value
    {
        value.clone_into(&mut text.0);
    }
}

#[expect(
    clippy::struct_excessive_bools,
    reason = "four independent facts of a button's state"
)]
#[derive(Clone, Copy)]
struct ButtonState {
    disabled: bool,
    pressed: bool,
    hovered: bool,
    on: bool,
}

const fn background_for(colors: &ThemeColors, state: ButtonState) -> Color {
    let ButtonState {
        disabled,
        pressed,
        hovered,
        on,
    } = state;
    if disabled {
        colors.background_primary
    } else if pressed {
        colors.selected_hovered_item
    } else if on {
        if hovered {
            colors.hovered_selected_item
        } else {
            colors.accent_subtle
        }
    } else if hovered {
        colors.hovered_item
    } else {
        colors.background_primary
    }
}

fn handle_hover(
    trigger: On<MouseEvent>,
    mut query: Query<
        (&mut BackgroundColor, Option<&ToggleState>),
        (Without<InteractionDisabled>, Without<NoDefaultHover>),
    >,
    theme: Res<Theme>,
) {
    // A `Drag` can arrive after `Out`; only the over, press, release and out events set hover.
    let kind = trigger.event().kind;
    if matches!(kind, MouseKind::Click(_) | MouseKind::Drag) {
        return;
    }
    let pressed = trigger.event().pressed();
    let hovered = trigger.event().hovered();
    if let Ok((mut background, toggle)) = query.get_mut(trigger.event().event_target()) {
        let on = toggle.is_some_and(|t| t.0);
        *background = BackgroundColor(background_for(
            theme.colors(),
            ButtonState {
                disabled: false,
                pressed,
                hovered,
                on,
            },
        ));
    }
}

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

fn restyle_on_size_change(
    changed: Query<
        Entity,
        (
            With<PushButtonRefs>,
            Or<(Changed<ButtonSize>, Changed<IconPlacement>)>,
        ),
    >,
    mut commands: Commands,
) {
    for entity in &changed {
        commands.run_system_cached_with(theme_push_button, Some(entity));
    }
}

fn restyle_on_hover_marker_change(
    added: Query<Entity, (With<PushButton>, Added<NoDefaultHover>)>,
    mut removed: RemovedComponents<NoDefaultHover>,
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
            &IconPlacement,
            Option<&ToggleState>,
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
        placement,
        toggle,
    ) in it
    {
        let on = toggle.is_some_and(|t| t.0);
        background_color.0 = background_for(
            theme.colors(),
            ButtonState {
                disabled,
                pressed: pressed && !no_default_hover,
                hovered: hovered.get() && !no_default_hover,
                on,
            },
        );
        border_color.set_all(theme.colors().border);
        node.border = UiRect::all(theme.border_thickness);
        node.flex_direction = placement.flex_direction();
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

        if let Ok((mut text_color, mut text_font)) = labels.get_mut(refs.label_id) {
            text_color.set_if_neq(TextColor(if disabled {
                theme.colors().text_disabled
            } else if on {
                theme.colors().accent_subtle_text
            } else {
                theme.colors().text_primary
            }));
            text_font.set_if_neq(TextFont {
                font: theme.body_font.clone().into(),
                font_size: Theme::font_size(font_size),
                weight,
                ..default()
            });
        }

        if let Some(icon_id) = refs.icon_id
            && let Ok((mut text_color, mut text_font)) = icons.get_mut(icon_id)
        {
            text_color.set_if_neq(TextColor(if disabled {
                theme.colors().text_disabled
            } else if on {
                theme.colors().accent_subtle_text
            } else {
                theme.colors().text_primary
            }));
            text_font.set_if_neq(TextFont {
                font: theme.body_font.clone().into(),
                font_size: Theme::font_size(font_size),
                weight,
                ..default()
            });
        }
    }
}
