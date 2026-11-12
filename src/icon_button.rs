//! `IconButton`: a borderless button showing only an icon, with an accessible name.

use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::{Activate, Button};
use itertools::Either;

use crate::focus::{AccessibilitySync, TabStop, sync_label, sync_toggled};
use crate::icon::{Icon, IconColor};
use crate::theme::{Theme, ThemeColors};
use crate::tooltip::Tooltip;
use crate::{MouseEvent, MouseKind};

/// Registers the systems and observers of [`IconButton`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct IconButtonPlugin;

impl Plugin for IconButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(restyle_on_toggle);
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_icon_buttons)
                    .run_if(resource_changed::<Theme>),
                restyle_on_disabled_change,
            ),
        )
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync));
    }
}

/// What an [`IconButton`] or a [`Tool`](crate::toolbar::Tool) of a
/// [`Toolbar`](crate::toolbar::Toolbar) means, which picks its colours.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum Tone {
    /// The ordinary colours.
    #[default]
    Neutral,
    /// A warning: the warning background, border and text.
    Warning,
    /// A dangerous action: the danger hover text, and the danger fill when on.
    Danger,
}

/// The sizes of an [`IconButton`].
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum IconButtonSize {
    /// A 44 px button with a 22 px icon.
    #[default]
    Regular,
    /// A 30 px button with a 16 px icon, whose icon is secondary text until the
    /// pointer is on the button.
    Small,
    /// A 54 px button with a 24 px icon.
    Large,
}

impl IconButtonSize {
    /// The side of the square button, in logical pixels.
    #[must_use]
    pub const fn button_px(self) -> f32 {
        match self {
            Self::Regular => 44.0,
            Self::Small => 30.0,
            Self::Large => 54.0,
        }
    }

    /// The side of the square icon, in logical pixels.
    #[must_use]
    pub const fn icon_px(self) -> f32 {
        match self {
            Self::Regular => 22.0,
            Self::Small => 16.0,
            Self::Large => 24.0,
        }
    }
}

/// A button that shows an [`Icon`] and nothing else: no border, the control
/// radius and the primary background, the hover background under the pointer.
///
/// Its [`Tone`] changes the colours. A [`Tone::Warning`] button has the warning
/// background, a 1 px warning border, a warning icon and a large shadow, is
/// brighter under the pointer, and while on (a [`ToggleState`], such as its panel being open) shows
/// a 3 px bar in the warning border colour. A [`Tone::Danger`] button turns its
/// icon the danger hover colour under the pointer and, when on, is filled with
/// the danger colour.
///
/// It is a [`bevy::ui_widgets::Button`], so an action observes
/// [`Activate`] on it, and it is disabled by
/// inserting [`InteractionDisabled`]. `name` is its accessible name, the label
/// of its [`AccessibilityNode`], since the icon has no text. The name is also
/// shown as a [`Tooltip`] when the pointer rests on the button.
///
/// Insert an `IconButton` again to change the icon (play and pause) or the name
/// of a live button.
///
/// The component is immutable: clone or build a value, edit it outside the world and insert it.
/// It cannot be queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::icon_button::IconButton;
/// fn edit(mut buttons: Query<&mut IconButton>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, PartialEq, Eq, Reflect, FromTemplate)]
#[reflect(Component)]
#[component(immutable, on_insert = Self::on_insert)]
#[require(
    Node {
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    BackgroundColor::default(),
    BorderColor::default(),
    BoxShadow::default(),
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
    /// What the button means; picks its colours.
    pub tone: Tone,
}

/// Whether a toggle [`IconButton`] is on, such as a pause button while the game
/// is paused. An on button shows the subtle accent background, an accent icon and
/// an accent bar along its bottom edge.
///
/// Insert it on an [`IconButton`] to make it a toggle: an
/// [`Activate`] (a click, or Enter / Space) inserts
/// the flipped value, which restyles the button and is what an app observes
/// (`On<Insert<ToggleState>>`). An app can insert a value itself to set it from
/// outside, for a keyboard shortcut or because the game paused itself. A button
/// without it is a plain, momentary button. It is one on/off button, not part of
/// a group. To a screen reader a button with it is a toggle button, pressed while
/// it is on; one without it has no toggled state at all.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ToggleState(pub bool);

/// The accent bar child of a toggle [`IconButton`], spawned with the first
/// [`ToggleState`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
struct ToggleBarRef(Entity);

/// The bar along the bottom edge of an on toggle button.
#[derive(Component, Clone, Copy, Debug)]
struct ToggleBar;

/// The height of the bar of an on toggle button, in logical pixels.
const BAR_HEIGHT_PX: f32 = 2.0;

/// The [`Icon`] child of an [`IconButton`], for reading. Only the widget builds it; the component
/// is immutable and its field is private.
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::icon_button::IconButtonRefs;
/// let _ = IconButtonRefs { icon_id: Entity::PLACEHOLDER };
/// ```
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
#[component(immutable)]
pub struct IconButtonRefs {
    icon_id: Entity,
}

impl IconButtonRefs {
    /// The icon.
    #[must_use]
    pub const fn icon(&self) -> Entity {
        self.icon_id
    }
}

impl IconButton {
    /// A button of `size` showing `icon`, named `name` for assistive tech.
    pub fn new(icon: Handle<Image>, name: impl Into<String>, size: IconButtonSize) -> Self {
        Self {
            icon,
            name: name.into(),
            size,
            tone: Tone::Neutral,
        }
    }

    /// This button with `tone`.
    #[must_use]
    pub const fn with_tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
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
                .observe(handle_hover)
                .observe(toggle_on_activate);
        }
        if let Some(button) = world.get::<Self>(entity) {
            let tooltip = Tooltip::new(button.name.clone());
            world.commands().entity(entity).insert(tooltip);
        }
        world
            .commands()
            .run_system_cached_with(theme_icon_buttons, Some(entity));
    }
}

/// What the pointer and the app are doing to a button.
#[expect(
    clippy::struct_excessive_bools,
    reason = "four independent facts of a button's state"
)]
#[derive(Copy, Clone, Debug, Default)]
struct ButtonState {
    disabled: bool,
    pressed: bool,
    hovered: bool,
    on: bool,
}

/// How a button looks.
#[derive(Copy, Clone, Debug, PartialEq)]
struct Look {
    background: Color,
    border: Option<Color>,
    icon: IconColor,
    /// The colour and height of the bar along the bottom edge, when it shows.
    bar: Option<(Color, f32)>,
}

/// The height of the bar of an on [`Tone::Warning`] button, in logical pixels.
const WARNING_BAR_HEIGHT_PX: f32 = 3.0;
/// How far a warning button moves from its background to its border colour under the pointer.
const WARNING_BRIGHTEN: f32 = 0.25;

/// The look of a button from its tone, size and state.
fn look(colors: &ThemeColors, tone: Tone, size: IconButtonSize, state: ButtonState) -> Look {
    let ButtonState {
        disabled,
        pressed,
        hovered,
        on,
    } = state;
    if disabled {
        return Look {
            background: colors.background_primary,
            border: None,
            icon: IconColor::TextDisabled,
            bar: None,
        };
    }
    match tone {
        Tone::Warning => {
            let background = if hovered || pressed || on {
                colors
                    .warning_background
                    .mix(&colors.warning_border, WARNING_BRIGHTEN)
            } else {
                colors.warning_background
            };
            Look {
                background,
                border: Some(colors.warning_border),
                icon: IconColor::WarningText,
                bar: on.then_some((colors.warning_border, WARNING_BAR_HEIGHT_PX)),
            }
        }
        Tone::Danger if on => Look {
            background: colors.danger,
            border: None,
            icon: IconColor::DangerText,
            bar: None,
        },
        Tone::Danger | Tone::Neutral => {
            let background = if on {
                colors.accent_subtle
            } else if pressed {
                colors.selected_hovered_item
            } else if hovered {
                colors.hovered_item
            } else {
                colors.background_primary
            };
            let icon = if on {
                IconColor::AccentSubtleText
            } else if tone == Tone::Danger && (hovered || pressed) {
                IconColor::DangerHoverText
            } else if size == IconButtonSize::Small && !hovered {
                IconColor::TextSecondary
            } else {
                IconColor::TextPrimary
            };
            Look {
                background,
                border: None,
                icon,
                bar: (on && tone == Tone::Neutral)
                    .then_some((colors.accent_subtle_bar, BAR_HEIGHT_PX)),
            }
        }
    }
}

/// Flips the [`ToggleState`] of a button that has one.
pub(crate) fn toggle_on_activate(
    trigger: On<Activate>,
    states: Query<&ToggleState, Without<InteractionDisabled>>,
    mut commands: Commands,
) {
    let target = trigger.event().event_target();
    if let Ok(state) = states.get(target) {
        commands.entity(target).insert(ToggleState(!state.0));
    }
}

/// Restyles a button whose [`ToggleState`] was just inserted.
fn restyle_on_toggle(t: On<Insert<ToggleState>>, mut commands: Commands) {
    commands.run_system_cached_with(theme_icon_buttons, Some(t.event().event_target()));
}

fn handle_hover(
    trigger: On<MouseEvent>,
    mut buttons: Query<
        (
            &IconButton,
            &IconButtonRefs,
            &mut BackgroundColor,
            &mut BorderColor,
            Option<&ToggleState>,
        ),
        Without<InteractionDisabled>,
    >,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let kind = trigger.event().kind;
    if matches!(kind, MouseKind::Click(_) | MouseKind::Drag) {
        return;
    }
    let pressed = matches!(kind, MouseKind::Pressed);
    let hovered = !matches!(kind, MouseKind::Out);
    let target = trigger.event().event_target();
    if let Ok((button, refs, mut background, mut border, toggle)) = buttons.get_mut(target) {
        let on = toggle.is_some_and(|t| t.0);
        let state = ButtonState {
            disabled: false,
            pressed,
            hovered,
            on,
        };
        let look = look(theme.colors(), button.tone, button.size, state);
        *background = BackgroundColor(look.background);
        border.set_all(look.border.unwrap_or(Color::NONE));
        commands
            .entity(refs.icon_id)
            .insert(Icon { color: look.icon });
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

/// Keeps the label of each button's [`AccessibilityNode`] at its name, and its
/// toggled state at its [`ToggleState`], or unset without one. Bevy clears the
/// label of a button without text when the button is added, so this runs after it.
fn sync_accessibility(
    mut buttons: ParamSet<(
        Query<
            (&IconButton, Option<&ToggleState>, &mut AccessibilityNode),
            Or<(
                Changed<AccessibilityNode>,
                Changed<IconButton>,
                Changed<ToggleState>,
            )>,
        >,
        Query<(&IconButton, Option<&ToggleState>, &mut AccessibilityNode)>,
    )>,
    mut untoggled: RemovedComponents<ToggleState>,
) {
    // Only buttons whose node Bevy reset, whose name changed or whose toggle was inserted are
    // looked at, plus those that lost their `ToggleState`; an idle frame touches none.
    for (button, toggle, mut node) in &mut buttons.p0() {
        sync_label(&mut node, &button.name);
        sync_toggled(&mut node, toggle.map(|toggle| toggle.0));
    }
    let mut all = buttons.p1();
    for entity in untoggled.read() {
        if let Ok((button, toggle, mut node)) = all.get_mut(entity) {
            sync_label(&mut node, &button.name);
            sync_toggled(&mut node, toggle.map(|toggle| toggle.0));
        }
    }
}

fn theme_icon_buttons(
    button_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut buttons: Query<(
        Entity,
        &IconButton,
        &IconButtonRefs,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut BoxShadow,
        &mut Node,
        Has<InteractionDisabled>,
        Has<Pressed>,
        &Hovered,
        Option<&ToggleState>,
        Option<&ToggleBarRef>,
    )>,
    mut bars: Query<(&mut Node, &mut BackgroundColor), (With<ToggleBar>, Without<IconButton>)>,
    icons: Query<(&Icon, &ImageNode, &Node), (Without<IconButton>, Without<ToggleBar>)>,
    mut commands: Commands,
) {
    let it = if let Some(button_id) = button_id.as_ref() {
        Either::Left(buttons.get_mut(*button_id).into_iter())
    } else {
        Either::Right(buttons.iter_mut())
    };
    for (
        entity,
        button,
        refs,
        mut background,
        mut border,
        mut shadow,
        mut node,
        disabled,
        pressed,
        hovered,
        toggle,
        bar,
    ) in it
    {
        let on = toggle.is_some_and(|t| t.0);
        let state = ButtonState {
            disabled,
            pressed,
            hovered: hovered.get(),
            on,
        };
        let Look {
            background: color,
            border: border_color,
            icon: icon_color,
            bar: bar_look,
        } = look(theme.colors(), button.tone, button.size, state);
        background.0 = color;
        border.set_all(border_color.unwrap_or(Color::NONE));
        node.border = if border_color.is_some() {
            UiRect::all(theme.border_thickness)
        } else {
            UiRect::ZERO
        };
        *shadow = if button.tone == Tone::Warning {
            BoxShadow::from(theme.large_shadow())
        } else {
            BoxShadow::default()
        };
        node.width = Val::Px(button.size.button_px());
        node.height = Val::Px(button.size.button_px());
        node.border_radius = BorderRadius::all(theme.control_radius);

        if toggle.is_some() {
            // Inset by the radius so the bar stays on the straight part of the edge.
            let bar_node = Node {
                position_type: PositionType::Absolute,
                left: theme.control_radius,
                right: theme.control_radius,
                bottom: Val::Px(0.0),
                height: Val::Px(bar_look.map_or(BAR_HEIGHT_PX, |(_, height)| height)),
                display: if bar_look.is_some() {
                    Display::Flex
                } else {
                    Display::None
                },
                ..default()
            };
            let bar_color = BackgroundColor(
                bar_look.map_or_else(|| theme.colors().accent_subtle_bar, |(color, _)| color),
            );
            match bar.and_then(|b| bars.get_mut(b.0).ok()) {
                Some((mut n, mut c)) => {
                    *n = bar_node;
                    *c = bar_color;
                }
                None if bar.is_none() => {
                    let id = commands
                        .spawn((
                            ToggleBar,
                            Name::new("ToggleBar"),
                            bar_node,
                            bar_color,
                            Pickable::IGNORE,
                            ChildOf(entity),
                        ))
                        .id();
                    commands.entity(entity).insert(ToggleBarRef(id));
                }
                None => {}
            }
        }

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
