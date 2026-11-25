//! `Toolbar`: a row of icon buttons over a [`Selection`](crate::select::Selection), in groups.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::entity::EntityHashSet;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use crate::divider::Divider;
use crate::focus::{
    AccessibilitySync, RovingGroup, RovingItem, TabStop, sync_label, sync_role, sync_toggled,
};
use crate::icon::{Icon, IconColor};
use crate::icon_button::{IconButtonSize, Tone};
use crate::select::{Item, SelectionParam, StructureChanges, order_views, plan_views};
use crate::theme::{Theme, ThemeColors};
use crate::tooltip::{OwnedTooltip, Tooltip};

const FILLED_WIDTH: f32 = 66.0;
const FILLED_HEIGHT: f32 = 60.0;
const FILLED_ICON: f32 = 22.0;
const ICON_ONLY_WIDTH: f32 = 56.0;
const ICON_ONLY_HEIGHT: f32 = 52.0;
const LABEL_GAP: f32 = 4.0;
const BUTTON_GAP: f32 = 4.0;
const BAR_HEIGHT: f32 = 2.0;
const HINT_FONT_SIZE: f32 = 10.0;
const HINT_OPACITY: f32 = 0.6;
const HINT_TOP: f32 = 5.0;
const HINT_RIGHT: f32 = 7.0;

/// Registers the systems and observers of [`Toolbar`].
#[derive(Copy, Clone, Debug)]
pub struct ToolbarPlugin;

impl Plugin for ToolbarPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item<Tool>>()
            .register_type::<Tool>()
            .register_type::<Toolbar>()
            .register_type::<ToolbarStyle>()
            .register_type::<ToolbarLabels>()
            .register_type::<ToolbarButton>()
            .add_systems(
                Update,
                (sync_buttons, update_buttons, sync_tooltips).chain(),
            )
            .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
            .add_observer(on_activate);
    }
}

/// How the active button of a [`Toolbar`] looks.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ToolbarStyle {
    /// Buttons with the icon above a small label; the active one is filled with the accent.
    #[default]
    Filled,
    /// Icon-only buttons; the active one has an accent bar along its bottom edge.
    Underlined,
}

/// Whether the buttons of a [`ToolbarStyle::Filled`] [`Toolbar`] show their label.
///
/// An [`ToolbarStyle::Underlined`] toolbar never shows labels, so this does not change it.
#[derive(Component, Debug, Copy, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum ToolbarLabels {
    /// The label is under the icon.
    #[default]
    Shown,
    /// The icon is alone.
    Hidden,
}

/// A row of icon buttons over a [`Selection`](crate::select::Selection) of [`Item<Tool>`]s.
///
/// Put it on an entity with a selection of [`Tool`] items, or a
/// [`Selection::Ref`](crate::select::Selection::Ref) to one. Items with different
/// [`Tool::group`]s are separated by a [`Divider`].
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("Toolbar"),
    ToolbarLabels,
    Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(BUTTON_GAP),
        ..default()
    },
    RovingGroup = RovingGroup::toolbar(),
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::Toolbar)),
)]
pub struct Toolbar {
    /// How the active button looks.
    pub style: ToolbarStyle,
}

impl Toolbar {
    /// A toolbar whose active button looks like `style`.
    #[must_use]
    pub const fn new(style: ToolbarStyle) -> Self {
        Self { style }
    }
}

/// What a button of a [`Toolbar`] shows: an icon and a label.
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct Tool {
    /// The icon image, white on transparent (see [`Icon`]).
    pub icon: Handle<Image>,
    /// The text under the icon of a [`ToolbarStyle::Filled`] toolbar; in every style the
    /// accessible name of the button.
    pub label: String,
    /// The group of the tool. A divider separates neighboring groups.
    pub group: u32,
    /// A shortcut key shown in the corner of the button. The app binds the key.
    pub key_hint: Option<String>,
    /// What the tool means, which sets its colors.
    pub tone: Tone,
}

impl Tool {
    /// This tool with `tone`.
    #[must_use]
    pub const fn with_tone(mut self, tone: Tone) -> Self {
        self.tone = tone;
        self
    }

    /// A tool in group 0.
    pub fn new(icon: Handle<Image>, label: impl Into<String>) -> Self {
        Self::in_group(icon, label, 0)
    }

    /// A tool in `group`.
    pub fn in_group(icon: Handle<Image>, label: impl Into<String>, group: u32) -> Self {
        Self {
            icon,
            label: label.into(),
            group,
            key_hint: None,
            tone: Tone::Neutral,
        }
    }

    /// This tool with `hint` shown in the corner of its button.
    #[must_use]
    pub fn with_key_hint(mut self, hint: impl Into<String>) -> Self {
        self.key_hint = Some(hint.into());
        self
    }
}

/// One button of a [`Toolbar`]: picks its item when it is activated.
///
/// Read its fields with [`item`](Self::item) and [`owner`](Self::owner).
///
/// ```compile_fail
/// use bevy_pluim::toolbar::ToolbarButton;
/// # let (item, toolbar) = (bevy::prelude::Entity::PLACEHOLDER, bevy::prelude::Entity::PLACEHOLDER);
/// let _ = ToolbarButton { item, owner: toolbar };
/// ```
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::toolbar::ToolbarButton;
/// fn change(mut buttons: Query<&mut ToolbarButton>) {}
/// bevy::ecs::system::assert_is_system(change);
/// ```
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ToolbarButton {
    item: Entity,
    owner: Entity,
}

impl ToolbarButton {
    pub(crate) const fn new(item: Entity, owner: Entity) -> Self {
        Self { item, owner }
    }

    /// The [`Item<Tool>`] this button shows.
    #[must_use]
    pub const fn item(&self) -> Entity {
        self.item
    }

    /// The [`Toolbar`] this button belongs to.
    #[must_use]
    pub const fn owner(&self) -> Entity {
        self.owner
    }
}

/// The children of a [`ToolbarButton`], by the style it was built for.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub enum ToolbarButtonRefs {
    /// A button of a [`ToolbarStyle::Filled`] toolbar.
    Filled {
        /// The [`Icon`].
        icon: Entity,
        /// The label text.
        label: Entity,
        /// The key hint text in the top-right corner; only for a tool with a [`Tool::key_hint`].
        hint: Option<Entity>,
    },
    /// A button of a [`ToolbarStyle::Underlined`] toolbar.
    Underlined {
        /// The [`Icon`].
        icon: Entity,
        /// The accent bar.
        bar: Entity,
        /// The key hint text in the top-right corner; only for a tool with a [`Tool::key_hint`].
        hint: Option<Entity>,
    },
}

impl ToolbarButtonRefs {
    /// The [`Icon`].
    #[must_use]
    pub const fn icon(&self) -> Entity {
        match *self {
            Self::Filled { icon, .. } | Self::Underlined { icon, .. } => icon,
        }
    }

    /// The key hint text, for a tool with a [`Tool::key_hint`].
    #[must_use]
    pub const fn hint(&self) -> Option<Entity> {
        match *self {
            Self::Filled { hint, .. } | Self::Underlined { hint, .. } => hint,
        }
    }

    /// The label text; only in a [`ToolbarStyle::Filled`] toolbar.
    #[must_use]
    pub const fn label(&self) -> Option<Entity> {
        match *self {
            Self::Filled { label, .. } => Some(label),
            Self::Underlined { .. } => None,
        }
    }

    /// The accent bar; only in a [`ToolbarStyle::Underlined`] toolbar.
    #[must_use]
    pub const fn bar(&self) -> Option<Entity> {
        match *self {
            Self::Underlined { bar, .. } => Some(bar),
            Self::Filled { .. } => None,
        }
    }

    /// The style the button was built for.
    #[must_use]
    pub const fn style(&self) -> ToolbarStyle {
        match self {
            Self::Filled { .. } => ToolbarStyle::Filled,
            Self::Underlined { .. } => ToolbarStyle::Underlined,
        }
    }
}

#[derive(Component, Debug, Copy, Clone)]
struct GroupDivider;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Slot {
    Button { item: Entity, hint: bool },
    Divider,
}

/// Brings the buttons and group dividers of every [`Toolbar`] in line with its items.
fn sync_buttons(
    mut changes: StructureChanges<Toolbar, Tool>,
    toolbars: Query<(&Toolbar, Option<&Children>)>,
    buttons: Query<(&ToolbarButton, &ToolbarButtonRefs)>,
    dividers: Query<(), With<GroupDivider>>,
    items: Query<&Item<Tool>>,
    selections: SelectionParam,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for toolbar_ref in changes.affected(&selections) {
        let Ok((toolbar, children)) = toolbars.get(toolbar_ref) else {
            continue;
        };
        let values = selections
            .get_items(toolbar_ref)
            .map_or(&[][..], |(_, values)| values);
        let mut wanted = Vec::new();
        let mut last_group = None;
        for item in values.iter().copied() {
            let Ok(tool) = items.get(item) else {
                continue;
            };
            if last_group.is_some_and(|group| group != tool.data.group) {
                wanted.push(Slot::Divider);
            }
            last_group = Some(tool.data.group);
            wanted.push(Slot::Button {
                item,
                hint: tool.data.key_hint.is_some(),
            });
        }

        let children = children
            .into_iter()
            .flat_map(RelationshipTarget::iter)
            .collect::<Vec<_>>();
        let plan = plan_views(
            children.iter().filter_map(|child| {
                if let Ok((button, refs)) = buttons.get(*child) {
                    let slot = Slot::Button {
                        item: button.item(),
                        hint: refs.hint().is_some(),
                    };
                    Some((*child, (refs.style() == toolbar.style).then_some(slot)))
                } else if dividers.contains(*child) {
                    Some((*child, Some(Slot::Divider)))
                } else {
                    None
                }
            }),
            &wanted,
        );
        let is_view = |child: Entity| buttons.contains(child) || dividers.contains(child);
        if plan.stale.is_empty()
            && plan.slots.iter().all(Option::is_some)
            && plan
                .slots
                .iter()
                .flatten()
                .copied()
                .eq(children.iter().copied().filter(|child| is_view(*child)))
        {
            continue;
        }
        for child in &plan.stale {
            commands.entity(*child).despawn();
        }
        let views = plan
            .slots
            .iter()
            .zip(&wanted)
            .map(|(kept, slot)| {
                kept.unwrap_or_else(|| match *slot {
                    Slot::Button { item, hint } => spawn_button(
                        &mut commands,
                        &theme,
                        toolbar_ref,
                        toolbar.style,
                        item,
                        hint,
                    ),
                    Slot::Divider => commands
                        .spawn((Divider::vertical(), GroupDivider, ChildOf(toolbar_ref)))
                        .id(),
                })
            })
            .collect::<Vec<_>>();
        if let Some(ordered) = order_views(&children, is_view, &views, &plan.stale) {
            commands.entity(toolbar_ref).replace_children(&ordered);
        }
    }
}

fn label_font(theme: &Theme) -> TextFont {
    TextFont {
        font: theme.body_font.clone().into(),
        font_size: Theme::font_size(theme.font_size_small),
        weight: FontWeight::MEDIUM,
        ..default()
    }
}

fn spawn_button(
    commands: &mut Commands,
    theme: &Theme,
    toolbar: Entity,
    style: ToolbarStyle,
    item: Entity,
    hint: bool,
) -> Entity {
    let (width, height, icon_px) = match style {
        ToolbarStyle::Filled => (FILLED_WIDTH, FILLED_HEIGHT, FILLED_ICON),
        ToolbarStyle::Underlined => (
            IconButtonSize::Regular.button_px(),
            IconButtonSize::Regular.button_px(),
            IconButtonSize::Regular.icon_px(),
        ),
    };
    // Reserve the entity first: a parentless `Node` becomes a `UiRoot`.
    let button = commands.spawn_empty().id();
    let icon = commands
        .spawn((
            Icon::bundle(Handle::default(), icon_px),
            Pickable::IGNORE,
            ChildOf(button),
        ))
        .id();
    let hint = hint.then(|| {
        commands
            .spawn((
                Name::new("ToolbarKeyHint"),
                Text::new(""),
                TextFont {
                    font: theme.body_font.clone().into(),
                    font_size: Theme::font_size(HINT_FONT_SIZE),
                    weight: FontWeight::SEMIBOLD,
                    ..default()
                },
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(HINT_TOP),
                    right: Val::Px(HINT_RIGHT),
                    ..default()
                },
                Pickable::IGNORE,
                ChildOf(button),
            ))
            .id()
    });
    let bar = |commands: &mut Commands| {
        commands
            .spawn((
                Name::new("ToolbarBar"),
                Node {
                    position_type: PositionType::Absolute,
                    left: theme.control_radius,
                    right: theme.control_radius,
                    bottom: Val::Px(0.0),
                    height: Val::Px(BAR_HEIGHT),
                    display: Display::None,
                    ..default()
                },
                BackgroundColor(theme.colors().accent_subtle_bar),
                Pickable::IGNORE,
                ChildOf(button),
            ))
            .id()
    };
    let refs = match style {
        ToolbarStyle::Filled => {
            let label = commands
                .spawn((
                    Text::new(""),
                    label_font(theme),
                    Pickable::IGNORE,
                    ChildOf(button),
                ))
                .id();
            ToolbarButtonRefs::Filled { icon, label, hint }
        }
        ToolbarStyle::Underlined => ToolbarButtonRefs::Underlined {
            icon,
            bar: bar(commands),
            hint,
        },
    };
    commands.entity(button).insert((
        Name::new("ToolbarButton"),
        ToolbarButton::new(item, toolbar),
        RovingItem::new(toolbar, item),
        refs,
        Button,
        Hovered::default(),
        TabStop,
        Node {
            width: Val::Px(width),
            height: Val::Px(height),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(LABEL_GAP),
            flex_shrink: 0.0,
            border_radius: BorderRadius::all(theme.control_radius),
            ..default()
        },
        BackgroundColor(Color::NONE),
        ChildOf(toolbar),
    ));
    button
}

#[derive(Debug, Copy, Clone, PartialEq)]
struct ButtonLook {
    background: Color,
    icon: IconColor,
    bar: bool,
}

/// The look of a button from its tone and state.
const fn button_look(
    colors: &ThemeColors,
    style: ToolbarStyle,
    tone: Tone,
    active: bool,
    hovered: bool,
) -> ButtonLook {
    if active {
        match tone {
            Tone::Danger => {
                return ButtonLook {
                    background: colors.danger,
                    icon: IconColor::DangerText,
                    bar: false,
                };
            }
            Tone::Warning => {
                return ButtonLook {
                    background: colors.warning_background,
                    icon: IconColor::WarningText,
                    bar: false,
                };
            }
            Tone::Neutral => {}
        }
    }
    let rest = if hovered {
        match tone {
            Tone::Warning => colors.warning_background,
            Tone::Danger | Tone::Neutral => colors.hovered_item,
        }
    } else {
        Color::NONE
    };
    match (style, active) {
        (ToolbarStyle::Filled, true) => ButtonLook {
            background: colors.accent,
            icon: IconColor::AccentText,
            bar: false,
        },
        (ToolbarStyle::Underlined, true) => ButtonLook {
            background: colors.accent_subtle,
            icon: IconColor::AccentSubtleText,
            bar: true,
        },
        (ToolbarStyle::Filled, false) => ButtonLook {
            background: rest,
            icon: match tone {
                Tone::Warning => IconColor::WarningText,
                Tone::Danger if hovered => IconColor::DangerHoverText,
                _ if hovered => IconColor::TextPrimary,
                _ => IconColor::TextSecondary,
            },
            bar: false,
        },
        (ToolbarStyle::Underlined, false) => ButtonLook {
            background: rest,
            icon: match tone {
                Tone::Warning => IconColor::WarningText,
                Tone::Danger if hovered => IconColor::DangerHoverText,
                _ => IconColor::TextPrimary,
            },
            bar: false,
        },
    }
}

/// Puts the icon, label, colors and sizes of every button in step with its item and state.
fn update_buttons(
    theme: Res<Theme>,
    toolbars: Query<(&Toolbar, &ToolbarLabels)>,
    selections: SelectionParam,
    items: Query<&Item<Tool>>,
    mut buttons: Query<(
        &ToolbarButton,
        Ref<ToolbarButtonRefs>,
        &Hovered,
        &mut BackgroundColor,
        &mut Node,
    )>,
    mut icons: Query<(&Icon, &mut ImageNode), Without<ToolbarButton>>,
    mut labels: Query<
        (&mut Text, &mut TextColor, &mut TextFont, &mut Node),
        Without<ToolbarButton>,
    >,
    mut bars: Query<
        (&mut Node, &mut BackgroundColor),
        (Without<ToolbarButton>, Without<Icon>, Without<Text>),
    >,
    mut commands: Commands,
) {
    for (button, refs, hovered, mut background, mut node) in &mut buttons {
        let Ok(item) = items.get(button.item()) else {
            continue;
        };
        let Ok((_, toolbar_labels)) = toolbars.get(button.owner()) else {
            continue;
        };
        let style = refs.style();
        let active = selections
            .get(button.owner())
            .is_some_and(|selection| selection.is_selected(button.item()));
        let look = button_look(theme.colors(), style, item.data.tone, active, hovered.get());
        let labels_shown = *toolbar_labels == ToolbarLabels::Shown;
        if style == ToolbarStyle::Filled {
            let (width, height) = if labels_shown {
                (FILLED_WIDTH, FILLED_HEIGHT)
            } else {
                (ICON_ONLY_WIDTH, ICON_ONLY_HEIGHT)
            };
            if node.width != Val::Px(width) {
                node.width = Val::Px(width);
            }
            if node.height != Val::Px(height) {
                node.height = Val::Px(height);
            }
        }
        background.set_if_neq(BackgroundColor(look.background));
        if theme.is_changed() {
            node.border_radius = BorderRadius::all(theme.control_radius);
        }

        if let Ok((icon, mut image)) = icons.get_mut(refs.icon()) {
            if image.image != item.data.icon {
                image.image = item.data.icon.clone();
            }
            if icon.color != look.icon {
                commands
                    .entity(refs.icon())
                    .insert(Icon { color: look.icon });
            }
        }
        if let Some(label) = refs.label()
            && let Ok((mut text, mut color, mut font, mut label_node)) = labels.get_mut(label)
        {
            let display = if labels_shown {
                Display::Flex
            } else {
                Display::None
            };
            if label_node.display != display {
                label_node.display = display;
            }
            if text.0 != item.data.label {
                item.data.label.clone_into(&mut text.0);
            }
            color.set_if_neq(TextColor(look.icon.color(theme.colors())));
            if theme.is_changed() || refs.is_added() {
                font.set_if_neq(label_font(&theme));
            }
        }
        if let Some(hint) = refs.hint()
            && let Ok((mut text, mut color, mut font, _)) = labels.get_mut(hint)
        {
            let key = item.data.key_hint.as_deref().unwrap_or_default();
            if text.0 != key {
                key.clone_into(&mut text.0);
            }
            color.set_if_neq(TextColor(
                look.icon.color(theme.colors()).with_alpha(HINT_OPACITY),
            ));
            if theme.is_changed() || refs.is_added() {
                let body_font = theme.body_font.clone().into();
                if font.font != body_font {
                    font.font = body_font;
                }
            }
        }
        if let Some(bar) = refs.bar()
            && let Ok((mut bar_node, mut bar_color)) = bars.get_mut(bar)
        {
            let display = if look.bar {
                Display::Flex
            } else {
                Display::None
            };
            if bar_node.display != display {
                bar_node.display = display;
            }
            if theme.is_changed() {
                bar_node.left = theme.control_radius;
                bar_node.right = theme.control_radius;
                bar_color.0 = theme.colors().accent_subtle_bar;
            }
        }
    }
}

/// Keeps the toolbar a toolbar and each button a toggle button. Run after Bevy's button sync.
fn sync_accessibility(
    mut toolbars: Query<
        &mut AccessibilityNode,
        (
            With<Toolbar>,
            Without<ToolbarButton>,
            Changed<AccessibilityNode>,
        ),
    >,
    mut buttons: ParamSet<(
        Query<
            (&ToolbarButton, &mut AccessibilityNode),
            Or<(Changed<AccessibilityNode>, Changed<ToolbarButton>)>,
        >,
        Query<(&ToolbarButton, &mut AccessibilityNode)>,
    )>,
    items: Query<&Item<Tool>>,
    selections: SelectionParam,
    mut changes: StructureChanges<Toolbar, Tool>,
) {
    for mut node in &mut toolbars {
        sync_role(&mut node, Role::Toolbar);
    }
    for (button, mut node) in &mut buttons.p0() {
        sync_button(button, &mut node, &items, &selections);
    }
    let affected: EntityHashSet = changes.affected(&selections).into_iter().collect();
    if affected.is_empty() {
        return;
    }
    for (button, mut node) in &mut buttons.p1() {
        if affected.contains(&button.owner()) {
            sync_button(button, &mut node, &items, &selections);
        }
    }
}

fn sync_button(
    button: &ToolbarButton,
    node: &mut Mut<AccessibilityNode>,
    items: &Query<&Item<Tool>>,
    selections: &SelectionParam,
) {
    let Ok(item) = items.get(button.item()) else {
        return;
    };
    let active = selections
        .get(button.owner())
        .is_some_and(|selection| selection.is_selected(button.item()));
    sync_label(node, &item.data.label);
    sync_toggled(node, Some(active));
}

/// Gives the buttons of an underlined toolbar a [`Tooltip`] with their label.
///
/// Only an [`OwnedTooltip`] is replaced or removed. An app tooltip with exactly the tool's label,
/// put on first, is not told apart from it.
fn sync_tooltips(
    buttons: Query<(
        Entity,
        &ToolbarButton,
        &ToolbarButtonRefs,
        Option<&Tooltip>,
        Option<&OwnedTooltip>,
    )>,
    items: Query<&Item<Tool>>,
    mut commands: Commands,
) {
    for (entity, button, refs, tooltip, owned) in &buttons {
        // A tooltip with other text is the app's: leave it.
        let owned = match (owned, tooltip) {
            (Some(owned), Some(tooltip)) if owned.0 == tooltip.text => true,
            (Some(_), _) => {
                commands.entity(entity).remove::<OwnedTooltip>();
                false
            }
            (None, _) => false,
        };
        if refs.style() != ToolbarStyle::Underlined {
            if owned {
                commands.entity(entity).remove::<(Tooltip, OwnedTooltip)>();
            }
            continue;
        }
        let Ok(item) = items.get(button.item()) else {
            continue;
        };
        let label = &item.data.label;
        if (tooltip.is_none() || owned) && tooltip.is_none_or(|t| t.text != *label) {
            commands
                .entity(entity)
                .insert((Tooltip::new(label.clone()), OwnedTooltip(label.clone())));
        }
    }
}

fn on_activate(t: On<Activate>, buttons: Query<&ToolbarButton>, mut selections: SelectionParam) {
    let Ok(button) = buttons.get(t.event().event_target()) else {
        return;
    };
    selections.update_selection(button.owner(), |selection| {
        selection.toggle_selected(button.item())
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS};

    #[test]
    fn the_active_filled_button_has_the_accent_background_and_accent_text() {
        for colors in [DEFAULT_LIGHT_THEME_COLORS, DEFAULT_DARK_THEME_COLORS] {
            let look = button_look(&colors, ToolbarStyle::Filled, Tone::Neutral, true, false);
            assert_eq!(look.background, colors.accent);
            assert_eq!(look.icon, IconColor::AccentText);
            assert!(!look.bar);
        }
    }

    #[test]
    fn the_active_underlined_button_has_the_subtle_accent_and_a_bar() {
        let colors = DEFAULT_DARK_THEME_COLORS;
        let look = button_look(&colors, ToolbarStyle::Underlined, Tone::Neutral, true, true);
        assert_eq!(look.background, colors.accent_subtle);
        assert_eq!(look.icon, IconColor::AccentSubtleText);
        assert!(look.bar);
    }

    #[test]
    fn a_hovered_button_that_is_not_active_has_the_hover_background() {
        let colors = DEFAULT_LIGHT_THEME_COLORS;
        for style in [ToolbarStyle::Filled, ToolbarStyle::Underlined] {
            assert_eq!(
                button_look(&colors, style, Tone::Neutral, false, true).background,
                colors.hovered_item
            );
            assert_eq!(
                button_look(&colors, style, Tone::Neutral, false, false).background,
                Color::NONE
            );
        }
    }
}
