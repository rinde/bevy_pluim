//! `Panel`: a card with a header, a close button and a body that grows to a
//! maximum height and then scrolls.

use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::card::Card;
use crate::divider::Divider;
use crate::focus::sync_label;
use crate::icon_button::{IconButton, IconButtonSize};
use crate::label::Label;
use crate::scroll_area::ScrollArea;

/// Registers the systems and observers of [`Panel`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct PanelPlugin;

impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sync_optional_sections);
    }
}

/// The width of a [`Panel`], in logical pixels.
pub const PANEL_WIDTH_PX: f32 = 380.0;

/// The default [`Panel::max_body_height`], in logical pixels.
pub const DEFAULT_MAX_BODY_HEIGHT_PX: f32 = 620.0;

/// A card with a header, a body that scrolls past a maximum height, and a footer.
///
/// The header holds a title, an optional subtitle beside it and an optional
/// close button, with an optional strip under it and a divider. The body grows
/// with its content up to [`Panel::max_body_height`] and then scrolls; an
/// optional footer sits above a divider.
///
/// The app spawns the contents of the body, the strip and the footer as
/// children of the entities in [`PanelRefs`]; a strip or footer without
/// children is not shown. The close button triggers [`PanelClosed`] on the
/// panel; the panel does not remove itself, the app decides what closing means.
///
/// Insert a `Panel` again to change the title, the subtitle, the close button or
/// the maximum height of a live panel.
///
/// To assistive technology it is a group, as a [`Card`] is, labelled with its title.
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Card = Card,
    Node {
        flex_direction: FlexDirection::Column,
        width: Val::Px(PANEL_WIDTH_PX),
        ..default()
    },
    Name::new("Panel"),
)]
pub struct Panel {
    /// The title, shown as a title [`Label`].
    pub title: String,
    /// Secondary text beside the title on the same baseline, such as "3 open".
    pub subtitle: Option<String>,
    /// The icon of the close button; with `None` the panel has no close button.
    pub close_icon: Option<Handle<Image>>,
    /// The height, in logical pixels, up to which the body grows before it scrolls.
    pub max_body_height: f32,
}

/// The entities of a [`Panel`], set when the panel is spawned.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanelRefs {
    /// The header row.
    pub header: Entity,
    /// The title [`Label`].
    pub title: Entity,
    /// The subtitle [`Label`], not shown while the panel has no subtitle.
    pub subtitle: Entity,
    /// The close [`IconButton`], not shown while the panel has no close icon.
    pub close: Entity,
    /// The strip under the header: spawn its contents as its children.
    pub strip: Entity,
    /// The [`ScrollArea`] around the body.
    pub scroll_area: Entity,
    /// The body: spawn its contents as its children.
    pub body: Entity,
    /// The footer: spawn its contents as its children.
    pub footer: Entity,
    footer_divider: Entity,
}

/// Triggered on a [`Panel`] when its close button is activated.
#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct PanelClosed {
    /// The panel to close.
    pub entity: Entity,
}

impl Panel {
    /// A panel titled `title` with no subtitle or close button and the default
    /// maximum body height.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            close_icon: None,
            max_body_height: DEFAULT_MAX_BODY_HEIGHT_PX,
        }
    }

    /// This panel with `subtitle` beside the title.
    #[must_use]
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// This panel with a close button showing `icon`.
    #[must_use]
    pub fn with_close_icon(mut self, icon: Handle<Image>) -> Self {
        self.close_icon = Some(icon);
        self
    }

    /// This panel whose body grows up to `max_body_height` logical pixels.
    #[must_use]
    pub const fn with_max_body_height(mut self, max_body_height: f32) -> Self {
        self.max_body_height = max_body_height;
        self
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let panel = context.entity;
        let has_parts = world.get::<PanelRefs>(panel).is_some();
        let mut commands = world.commands();
        if !has_parts {
            spawn_parts(&mut commands, panel);
        }
        commands.run_system_cached_with(sync_panel, panel);
    }
}

fn spawn_parts(commands: &mut Commands, panel: Entity) {
    // The body is reserved now so the scroll area's closure can name it.
    let body = commands.spawn_empty().id();
    let header = commands
        .spawn((
            Name::new("PanelHeader"),
            Node {
                align_items: AlignItems::Baseline,
                padding: UiRect {
                    top: Val::Px(14.0),
                    right: Val::Px(12.0),
                    bottom: Val::Px(10.0),
                    left: Val::Px(18.0),
                },
                ..default()
            },
            ChildOf(panel),
        ))
        .id();
    let title = commands
        .spawn((Label::title(""), Node::default(), ChildOf(header)))
        .id();
    let subtitle = commands
        .spawn((
            Label::secondary(""),
            Node {
                margin: UiRect::left(Val::Px(8.0)),
                ..default()
            },
            ChildOf(header),
        ))
        .id();
    let close = commands
        .spawn((
            IconButton::new(Handle::default(), "Close", IconButtonSize::Small),
            Node {
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
                margin: UiRect::left(Val::Auto),
                ..default()
            },
            ChildOf(header),
        ))
        .observe(move |_: On<Activate>, mut commands: Commands| {
            commands.trigger(PanelClosed { entity: panel });
        })
        .id();
    let strip = commands
        .spawn((
            Name::new("PanelStrip"),
            Node {
                flex_wrap: FlexWrap::Wrap,
                padding: UiRect {
                    top: Val::ZERO,
                    right: Val::Px(12.0),
                    bottom: Val::Px(10.0),
                    left: Val::Px(18.0),
                },
                display: Display::None,
                ..default()
            },
            ChildOf(panel),
        ))
        .id();
    commands.spawn((Divider::horizontal(), ChildOf(panel)));
    let scroll_area = commands
        .spawn((
            ScrollArea::bundle(
                move |s| {
                    let contents = s.target_entity();
                    s.world_mut().entity_mut(body).insert((
                        Name::new("PanelBody"),
                        Node {
                            flex_direction: FlexDirection::Column,
                            width: Val::Percent(100.0),
                            flex_shrink: 0.0,
                            ..default()
                        },
                        // The wheel scrolls the node it hits, so the body (which covers the
                        // contents) lets the pointer through to them.
                        Pickable {
                            should_block_lower: false,
                            ..default()
                        },
                        ChildOf(contents),
                    ));
                },
                true,
                false,
            ),
            Node {
                width: Val::Percent(100.0),
                ..default()
            },
            ChildOf(panel),
        ))
        .id();
    let footer_divider = commands.spawn((Divider::horizontal(), ChildOf(panel))).id();
    let footer = commands
        .spawn((
            Name::new("PanelFooter"),
            Node {
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(Val::Px(18.0), Val::Px(10.0)),
                display: Display::None,
                ..default()
            },
            ChildOf(panel),
        ))
        .id();
    commands.entity(panel).insert(PanelRefs {
        header,
        title,
        subtitle,
        close,
        strip,
        scroll_area,
        body,
        footer,
        footer_divider,
    });
}

/// Puts a panel's title, subtitle, close button and maximum height on its parts, and its
/// title on its accessibility node.
fn sync_panel(
    panel: In<Entity>,
    mut panels: Query<(&Panel, &PanelRefs, Option<&mut AccessibilityNode>)>,
    mut nodes: Query<&mut Node>,
    mut commands: Commands,
) {
    let Ok((panel, refs, accessibility_node)) = panels.get_mut(*panel) else {
        return;
    };
    if let Some(mut accessibility_node) = accessibility_node {
        sync_label(&mut accessibility_node, &panel.title);
    }
    let show = |shown: bool| if shown { Display::Flex } else { Display::None };
    commands
        .entity(refs.title)
        .insert(Label::title(panel.title.clone()));
    commands
        .entity(refs.subtitle)
        .insert(Label::secondary(panel.subtitle.clone().unwrap_or_default()));
    commands.entity(refs.close).insert(IconButton::new(
        panel.close_icon.clone().unwrap_or_default(),
        "Close",
        IconButtonSize::Small,
    ));
    if let Ok(mut node) = nodes.get_mut(refs.subtitle) {
        node.display = show(panel.subtitle.is_some());
    }
    if let Ok(mut node) = nodes.get_mut(refs.close) {
        node.display = show(panel.close_icon.is_some());
    }
    if let Ok(mut node) = nodes.get_mut(refs.scroll_area) {
        node.max_height = Val::Px(panel.max_body_height);
    }
}

/// Shows the strip and the footer (and the divider above it) only while they
/// have children.
fn sync_optional_sections(
    panels: Query<&PanelRefs>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
) {
    for refs in &panels {
        for (section, divider) in [(refs.strip, None), (refs.footer, Some(refs.footer_divider))] {
            let display = if children.get(section).is_ok_and(|c| !c.is_empty()) {
                Display::Flex
            } else {
                Display::None
            };
            for entity in std::iter::once(section).chain(divider) {
                if let Ok(mut node) = nodes.get_mut(entity)
                    && node.display != display
                {
                    node.display = display;
                }
            }
        }
    }
}
