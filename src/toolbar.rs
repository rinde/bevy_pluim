//! `Toolbar`: a row of icon buttons over a [`Selection`](crate::select::Selection), in groups.

use bevy::a11y::AccessibilityNode;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use crate::divider::Divider;
use crate::focus::TabStop;
use crate::icon::{Icon, IconColor};
use crate::icon_button::IconButtonSize;
use crate::select::{Item, SelectionParam};
use crate::theme::{Theme, ThemeColors};
use crate::tooltip::Tooltip;

/// The width of a button of a [`ToolbarStyle::Filled`] toolbar, in logical pixels.
const FILLED_WIDTH: f32 = 66.0;
/// The height of a button of a [`ToolbarStyle::Filled`] toolbar.
const FILLED_HEIGHT: f32 = 60.0;
/// The side of the icon of a button of a [`ToolbarStyle::Filled`] toolbar.
const FILLED_ICON: f32 = 22.0;
/// The width of a button of a [`ToolbarStyle::Filled`] toolbar whose labels are hidden.
const ICON_ONLY_WIDTH: f32 = 56.0;
/// The height of a button of a [`ToolbarStyle::Filled`] toolbar whose labels are hidden.
const ICON_ONLY_HEIGHT: f32 = 52.0;
/// The gap between the icon and the label of a button.
const LABEL_GAP: f32 = 4.0;
/// The gap between two buttons, and between a button and a group divider.
const BUTTON_GAP: f32 = 4.0;
/// The height of the bar along the bottom edge of the active button of a
/// [`ToolbarStyle::Underlined`] toolbar.
const BAR_HEIGHT: f32 = 2.0;

/// Registers the systems and observers of [`Toolbar`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
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
                (
                    sync_buttons,
                    update_buttons,
                    sync_accessible_names,
                    sync_tooltips,
                )
                    .chain(),
            )
            .add_observer(on_activate);
    }
}

/// How the active button of a [`Toolbar`] looks.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ToolbarStyle {
    /// 66 by 60 px buttons with the icon above a small label; the active one is filled with the
    /// accent, and its icon and label take the accent text colour. Like a tool dock.
    #[default]
    Filled,
    /// 44 px buttons with an icon only (the label is the accessible name); the active one has
    /// the subtle accent background, an accent icon and an accent bar along its bottom edge.
    /// Like a panel bar.
    Underlined,
}

/// Whether the buttons of a [`ToolbarStyle::Filled`] [`Toolbar`] show their label.
///
/// Insert a new value on the toolbar to switch; the buttons are laid out again, not respawned.
/// With the labels hidden the buttons are 56 by 52 px and show the icon alone; the label stays
/// the accessible name. An [`ToolbarStyle::Underlined`] toolbar never shows labels, so this does
/// not change it.
#[derive(Component, Debug, Copy, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub enum ToolbarLabels {
    /// The label is under the icon.
    #[default]
    Shown,
    /// The icon is alone.
    Hidden,
}

/// A row of icon buttons over a [`Selection`](crate::select::Selection) of [`Item<Tool>`]s, where
/// picking a button selects its item.
///
/// Put it on an entity together with a selection that has [`Tool`] items, such as
/// `Selection::builder().one().build_with_items([Item { data: Tool::new(icon, "Rail") }])`, or
/// with a [`Selection::Ref`](crate::select::Selection::Ref) to one, so that a keyboard shortcut
/// and a click go through the same selection. With a `One` selection exactly one tool is active;
/// with a `ZeroOrOne` selection activating the active tool clears the selection.
///
/// Items with different [`Tool::group`]s are separated by a vertical [`Divider`]. A button is a
/// [`Button`] and a tab stop: Enter and Space on a focused one pick it, like a click.
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
)]
pub struct Toolbar {
    /// How the active button looks, and with it the size of the buttons and whether they show
    /// their label.
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
    /// The group of the tool. A divider is put between neighbouring tools whose groups differ.
    pub group: u32,
}

impl Tool {
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
        }
    }
}

/// One button of a [`Toolbar`]: picks its item when it is activated.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct ToolbarButton {
    /// The [`Item<Tool>`] this button shows.
    pub item: Entity,
    /// The [`Toolbar`] this button belongs to.
    pub toolbar: Entity,
    /// The style the button was built for.
    pub style: ToolbarStyle,
}

/// The children of a [`ToolbarButton`].
#[derive(Component, Debug, Copy, Clone)]
pub struct ToolbarButtonRefs {
    /// The [`Icon`].
    pub icon: Entity,
    /// The label text; only in a [`ToolbarStyle::Filled`] toolbar.
    pub label: Option<Entity>,
    /// The accent bar; only in a [`ToolbarStyle::Underlined`] toolbar.
    pub bar: Option<Entity>,
}

/// A divider between two groups of a [`Toolbar`].
#[derive(Component, Debug, Copy, Clone)]
struct GroupDivider;

/// What a child of a toolbar is, to tell whether the children match the items.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Slot {
    Button(Entity),
    Divider,
}

/// Spawns the buttons and group dividers of every [`Toolbar`] again when its items, their
/// groups or its style no longer match the children.
fn sync_buttons(
    toolbars: Query<(Entity, &Toolbar, Option<&Children>)>,
    buttons: Query<&ToolbarButton>,
    dividers: Query<(), With<GroupDivider>>,
    items: Query<&Item<Tool>>,
    selections: SelectionParam,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for (toolbar_ref, toolbar, children) in &toolbars {
        let Some((_, values)) = selections.get_item_values(toolbar_ref) else {
            continue;
        };
        let mut wanted = Vec::new();
        let mut last_group = None;
        for item in values.iter() {
            let Ok(tool) = items.get(item) else {
                continue;
            };
            if last_group.is_some_and(|group| group != tool.data.group) {
                wanted.push(Slot::Divider);
            }
            last_group = Some(tool.data.group);
            wanted.push(Slot::Button(item));
        }

        let mut existing = Vec::new();
        let mut current_style = true;
        for child in children.into_iter().flat_map(RelationshipTarget::iter) {
            if let Ok(button) = buttons.get(child) {
                existing.push((child, Slot::Button(button.item)));
                current_style &= button.style == toolbar.style;
            } else if dividers.contains(child) {
                existing.push((child, Slot::Divider));
            }
        }
        if current_style
            && existing
                .iter()
                .map(|(_, slot)| *slot)
                .eq(wanted.iter().copied())
        {
            continue;
        }
        for (child, _) in existing {
            commands.entity(child).despawn();
        }
        for slot in wanted {
            match slot {
                Slot::Button(item) => {
                    spawn_button(&mut commands, &theme, toolbar_ref, toolbar.style, item);
                }
                Slot::Divider => {
                    commands.spawn((Divider::vertical(), GroupDivider, ChildOf(toolbar_ref)));
                }
            }
        }
    }
}

fn label_font(theme: &Theme) -> TextFont {
    TextFont {
        font: theme.body_font.clone().into(),
        font_size: FontSize::Px(theme.font_size_small),
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
) {
    let (width, height, icon_px) = match style {
        ToolbarStyle::Filled => (FILLED_WIDTH, FILLED_HEIGHT, FILLED_ICON),
        ToolbarStyle::Underlined => (
            IconButtonSize::Regular.button_px(),
            IconButtonSize::Regular.button_px(),
            IconButtonSize::Regular.icon_px(),
        ),
    };
    let icon = commands
        .spawn((Icon::bundle(Handle::default(), icon_px), Pickable::IGNORE))
        .id();
    let label = (style == ToolbarStyle::Filled).then(|| {
        commands
            .spawn((Text::new(""), label_font(theme), Pickable::IGNORE))
            .id()
    });
    let bar = (style == ToolbarStyle::Underlined).then(|| {
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
                BackgroundColor(theme.colors.accent_subtle_bar),
                Pickable::IGNORE,
            ))
            .id()
    });
    let button = commands
        .spawn((
            Name::new("ToolbarButton"),
            ToolbarButton {
                item,
                toolbar,
                style,
            },
            ToolbarButtonRefs { icon, label, bar },
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
        ))
        .id();
    commands.entity(button).add_children(&[icon]);
    if let Some(label) = label {
        commands.entity(button).add_children(&[label]);
    }
    if let Some(bar) = bar {
        commands.entity(button).add_children(&[bar]);
    }
}

/// How a button looks.
#[derive(Debug, Copy, Clone, PartialEq)]
struct ButtonLook {
    background: Color,
    icon: IconColor,
    bar: bool,
}

/// The look of a button from its state: active, hovered or at rest. Used by the system that
/// restyles buttons on every change, so a theme change under a resting pointer keeps the hover.
const fn button_look(
    colors: &ThemeColors,
    style: ToolbarStyle,
    active: bool,
    hovered: bool,
) -> ButtonLook {
    let rest = if hovered {
        colors.hovered_item
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
            icon: if hovered {
                IconColor::TextPrimary
            } else {
                IconColor::TextSecondary
            },
            bar: false,
        },
        (ToolbarStyle::Underlined, false) => ButtonLook {
            background: rest,
            icon: IconColor::TextPrimary,
            bar: false,
        },
    }
}

/// Puts the icon, label, colours and sizes of every button in step with its item, its
/// selection, the pointer and the theme.
fn update_buttons(
    theme: Res<Theme>,
    toolbars: Query<&ToolbarLabels>,
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
        let Ok(item) = items.get(button.item) else {
            continue;
        };
        let active = selections
            .get(button.toolbar)
            .is_some_and(|selection| selection.is_selected(button.item));
        let look = button_look(&theme.colors, button.style, active, hovered.get());
        let labels_shown = toolbars
            .get(button.toolbar)
            .is_ok_and(|labels| *labels == ToolbarLabels::Shown);
        if button.style == ToolbarStyle::Filled {
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

        if let Ok((icon, mut image)) = icons.get_mut(refs.icon) {
            if image.image != item.data.icon {
                image.image = item.data.icon.clone();
            }
            if icon.color != look.icon {
                commands.entity(refs.icon).insert(Icon { color: look.icon });
            }
        }
        if let Some(label) = refs.label
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
            color.set_if_neq(TextColor(look.icon.color(&theme.colors)));
            if theme.is_changed() || refs.is_added() {
                *font = label_font(&theme);
            }
        }
        if let Some(bar) = refs.bar
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
                bar_color.0 = theme.colors.accent_subtle_bar;
            }
        }
    }
}

/// Keeps the label of each button's [`AccessibilityNode`] at the label of its tool. Bevy
/// clears the label of a button that has no text of its own when it is added, so this also
/// sets it again after that.
fn sync_accessible_names(
    mut buttons: Query<(&ToolbarButton, &mut AccessibilityNode)>,
    items: Query<&Item<Tool>>,
) {
    for (button, mut node) in &mut buttons {
        let Ok(item) = items.get(button.item) else {
            continue;
        };
        if node.label() != Some(item.data.label.as_str()) {
            node.set_label(item.data.label.as_str());
        }
    }
}

/// Gives the buttons of an underlined toolbar, whose labels are hidden, a [`Tooltip`] with the
/// label of their tool.
fn sync_tooltips(
    buttons: Query<(Entity, &ToolbarButton, Option<&Tooltip>)>,
    items: Query<&Item<Tool>>,
    mut commands: Commands,
) {
    for (entity, button, tooltip) in &buttons {
        if button.style != ToolbarStyle::Underlined {
            continue;
        }
        let Ok(item) = items.get(button.item) else {
            continue;
        };
        if tooltip.is_none_or(|t| t.text != item.data.label) {
            commands
                .entity(entity)
                .insert(Tooltip::new(item.data.label.clone()));
        }
    }
}

/// Picks the tool of the button that was used (a click, or Enter or Space on it).
fn on_activate(t: On<Activate>, buttons: Query<&ToolbarButton>, mut selections: SelectionParam) {
    let Ok(button) = buttons.get(t.event().event_target()) else {
        return;
    };
    selections.update_selection(button.toolbar, |selection| {
        selection.toggle_selected(button.item)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS};

    #[test]
    fn the_active_filled_button_has_the_accent_background_and_accent_text() {
        for colors in [DEFAULT_LIGHT_THEME_COLORS, DEFAULT_DARK_THEME_COLORS] {
            let look = button_look(&colors, ToolbarStyle::Filled, true, false);
            assert_eq!(look.background, colors.accent);
            assert_eq!(look.icon, IconColor::AccentText);
            assert!(!look.bar);
        }
    }

    #[test]
    fn the_active_underlined_button_has_the_subtle_accent_and_a_bar() {
        let colors = DEFAULT_DARK_THEME_COLORS;
        let look = button_look(&colors, ToolbarStyle::Underlined, true, true);
        assert_eq!(look.background, colors.accent_subtle);
        assert_eq!(look.icon, IconColor::AccentSubtleText);
        assert!(look.bar);
    }

    #[test]
    fn a_hovered_button_that_is_not_active_has_the_hover_background() {
        let colors = DEFAULT_LIGHT_THEME_COLORS;
        for style in [ToolbarStyle::Filled, ToolbarStyle::Underlined] {
            assert_eq!(
                button_look(&colors, style, false, true).background,
                colors.hovered_item
            );
            assert_eq!(
                button_look(&colors, style, false, false).background,
                Color::NONE
            );
        }
    }
}
