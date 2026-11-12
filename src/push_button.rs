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
use crate::focus::{AccessibilitySync, TabStop, sync_label, sync_toggled};
use crate::icon_button::{ToggleState, toggle_on_activate};
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
                restyle_on_hover_marker_change,
            ),
        )
        .add_observer(restyle_on_toggle)
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync));
    }
}

/// Restyles a [`PushButton`] whose [`ToggleState`] was just inserted.
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

/// Keeps the label of each button's [`AccessibilityNode`] at its text, without the icon, and its
/// toggled state at its [`ToggleState`] (unset without one). Bevy leaves a
/// [`bevy::ui_widgets::Button`] unlabelled. A combo's button is left alone: it is a
/// combo box whose value says the selection.
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
    // Only buttons whose node Bevy reset, whose text changed or whose toggle was inserted are
    // looked at, plus those that lost their `ToggleState`; an idle frame touches none.
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
///
/// It is a [`bevy::ui_widgets::Button`]: it is [`Pressed`] while
/// held, triggers [`Activate`](bevy::ui_widgets::Activate) when it is used (a
/// click, or Enter or Space on a focused button) and is disabled by inserting
/// [`InteractionDisabled`]. React to a use of the button by observing
/// `Activate`; the hover and press colours follow the pointer on their own, and
/// stay right when the [`Theme`] changes under a pointer that rests on the button.
/// Its [`Hovered`] says whether the pointer is on it.
///
/// Insert a [`ToggleState`] to make it an on/off toggle, as on an
/// [`IconButton`](crate::icon_button::IconButton): an `Activate` inserts the flipped value
/// (observe `On<Insert<ToggleState>>` to react), and an on button has the `accent_subtle`
/// background and `accent_subtle_text` text. To a screen reader it is `toggled` with the state;
/// without a `ToggleState` it has no toggled state.
///
/// Insert a `PushButton` again to change the label or add or remove the icon of a live button:
/// the existing text children are updated in place (the icon child is spawned or despawned and
/// [`PushButtonRefs`] follows), so the text on screen always equals the accessible label.
///
/// The component is immutable: its configuration is read when it is inserted, so clone or build
/// a value, edit it outside the world and insert it again. It cannot be queried mutably:
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

/// Where the icon of a [`PushButton`] sits relative to its label.
///
/// Insert it with the button, or later; it only moves the existing children, nothing is
/// respawned. A button without an icon is not affected.
///
/// The default is `Right`, the order the button has always had.
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
    /// The flex direction that lays out `[label, icon]` (the order they are spawned in) so the
    /// icon ends up where this placement says.
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

/// The children of a [`PushButton`], for reading. Only the widget builds and updates it; the
/// component is immutable and its fields are private.
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

/// Marker component that disables the default hover effect for `PushButton`s.
///
/// Inserting or removing it on a live button takes effect at once: the button rests at `background_primary`
/// for its owner to colour, or follows the pointer again.
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

    /// Builds the children and observers once, and on every later insert brings the label and the
    /// icon of the existing children in line with the new value. Runs as one queued command so
    /// two inserts before a flush cannot both spawn children.
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

/// Sets the string of the `Text` on `entity`, only when it differs.
fn set_text(world: &mut World, entity: Entity, value: &str) {
    if let Some(mut text) = world.get_mut::<Text>(entity)
        && text.0 != value
    {
        value.clone_into(&mut text.0);
    }
}

/// What the pointer and the app are doing to a [`PushButton`].
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

/// The background of a [`PushButton`] with the default hover, from its state.
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

/// Restyles a [`PushButton`] whose [`ButtonSize`] or [`IconPlacement`] was changed after it was
/// spawned.
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

/// Restyles a [`PushButton`] that gained or lost [`NoDefaultHover`].
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
        // A button with `NoDefaultHover` is coloured by its owner, so it rests here.
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

        // label
        if let Ok((mut text_color, mut text_font)) = labels.get_mut(refs.label_id) {
            *text_color = TextColor(if disabled {
                theme.colors().text_disabled
            } else if on {
                theme.colors().accent_subtle_text
            } else {
                theme.colors().text_primary
            });
            *text_font = TextFont {
                font: theme.body_font.clone().into(),
                font_size: Theme::font_size(font_size),
                weight,
                ..default()
            };
        }

        // icon
        if let Some(icon_id) = refs.icon_id
            && let Ok((mut text_color, mut text_font)) = icons.get_mut(icon_id)
        {
            *text_color = TextColor(if disabled {
                theme.colors().text_disabled
            } else if on {
                theme.colors().accent_subtle_text
            } else {
                theme.colors().text_primary
            });
            *text_font = TextFont {
                font: theme.body_font.clone().into(),
                font_size: Theme::font_size(font_size),
                weight,
                ..default()
            };
        }
    }
}
