//! `Panel`: a card with a header (title, subtitle, close button), optional strip and
//! footer, and a body that grows with its content up to a maximum height, then scrolls.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::card::Card;
use crate::divider::Divider;
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
        app.add_systems(Update, show_slots_with_content);
    }
}

/// The width of a [`Panel`], in logical pixels.
pub const PANEL_WIDTH_PX: f32 = 380.0;

/// The default [`Panel::max_body_height`], in logical pixels.
pub const PANEL_MAX_BODY_HEIGHT_PX: f32 = 620.0;

/// The padding of the header: 14 px above, 12 px right, 10 px below, 18 px left.
const HEADER_PADDING: UiRect = UiRect {
    top: Val::Px(14.0),
    right: Val::Px(12.0),
    bottom: Val::Px(10.0),
    left: Val::Px(18.0),
};

/// The padding of the strip and the footer.
const SLOT_PADDING: UiRect = UiRect {
    top: Val::Px(8.0),
    right: Val::Px(18.0),
    bottom: Val::Px(8.0),
    left: Val::Px(18.0),
};

/// A card with a header, an optional strip, a divider, a scrolling body and an
/// optional footer, as the information panels of a game share.
///
/// Spawn it with [`Panel::new`], then spawn the app's content as children of
/// the entities in [`PanelRefs`]: [`body`](PanelRefs::body), and optionally
/// [`strip`](PanelRefs::strip) (under the header, such as filter chips) and
/// [`footer`](PanelRefs::footer) (under the body, above a divider). The strip
/// and the footer show only while they have children.
///
/// The body grows with its content up to [`max_body_height`](Self::max_body_height)
/// and scrolls beyond that (it is a [`ScrollArea`]).
///
/// A panel with a [`close_icon`](Self::close_icon) has a close button, which
/// triggers [`PanelClosed`] on the panel; the app observes it, typically to clear
/// a selection, and decides what closing means. Insert the `Panel` again to
/// change the title, the subtitle, the icon or the maximum height of a live panel.
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
#[require(
    Card = Card,
    Node {
        width: Val::Px(PANEL_WIDTH_PX),
        flex_direction: FlexDirection::Column,
        ..default()
    },
    Name::new("Panel"),
)]
pub struct Panel {
    /// The title, shown in the title [`Label`].
    pub title: String,
    /// The secondary text beside the title, such as "4 on the network".
    pub subtitle: Option<String>,
    /// The icon of the close button (see [`IconButton`]); without one the panel has
    /// no close button.
    pub close_icon: Option<Handle<Image>>,
    /// The height the body grows to before it scrolls, in logical pixels.
    pub max_body_height: f32,
}

/// The parts of a [`Panel`], spawned with it.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PanelRefs {
    /// The header row.
    pub header: Entity,
    /// The title [`Label`].
    pub title: Entity,
    /// The subtitle [`Label`], hidden while the panel has none.
    pub subtitle: Entity,
    /// The close [`IconButton`], hidden while the panel has no close icon.
    pub close: Entity,
    /// The container under the header for the app's content, hidden while empty.
    pub strip: Entity,
    /// The [`ScrollArea`] around the body.
    pub scroll_area: Entity,
    /// The container for the body's content, inside the scroll area.
    pub body: Entity,
    /// The container above the footer's divider for the app's content, hidden
    /// while empty.
    pub footer: Entity,
    /// The divider above the footer, hidden while the footer is.
    pub footer_divider: Entity,
}

/// Triggered on a [`Panel`] when its close button is activated.
///
/// The panel does not hide or despawn itself: the app decides what closing means.
#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct PanelClosed {
    /// The panel.
    pub entity: Entity,
}

impl Panel {
    /// A panel with `title`, no subtitle and no close button.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            close_icon: None,
            max_body_height: PANEL_MAX_BODY_HEIGHT_PX,
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

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let panel = context.entity;
        if world.get::<PanelRefs>(panel).is_none() {
            let mut commands = world.commands();
            let header = commands.spawn_empty().id();
            let title = commands.spawn_empty().id();
            let subtitle = commands.spawn_empty().id();
            let close = commands.spawn_empty().id();
            let strip = commands.spawn_empty().id();
            let scroll_area = commands.spawn_empty().id();
            let body = commands.spawn_empty().id();
            let footer = commands.spawn_empty().id();
            let footer_divider = commands.spawn_empty().id();

            commands.entity(header).insert((
                Name::new("PanelHeader"),
                Node {
                    flex_direction: FlexDirection::Row,
                    align_items: AlignItems::Center,
                    column_gap: Val::Px(8.0),
                    padding: HEADER_PADDING,
                    ..default()
                },
                ChildOf(panel),
            ));
            let title_group = commands
                .spawn((
                    Name::new("PanelTitles"),
                    Node {
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Baseline,
                        column_gap: Val::Px(8.0),
                        flex_grow: 1.0,
                        min_width: Val::Px(0.0),
                        ..default()
                    },
                    ChildOf(header),
                ))
                .id();
            let beside_title = Node {
                align_self: AlignSelf::Auto,
                ..default()
            };
            commands.entity(title).insert((
                Label::title(""),
                beside_title.clone(),
                ChildOf(title_group),
            ));
            commands.entity(subtitle).insert((
                Label::secondary(""),
                beside_title,
                ChildOf(title_group),
            ));
            commands.entity(close).insert((
                IconButton::new(Handle::default(), "Close", IconButtonSize::Small),
                ChildOf(header),
            ));
            commands
                .entity(close)
                .observe(move |_: On<Activate>, mut commands: Commands| {
                    commands.trigger(PanelClosed { entity: panel });
                });

            commands.entity(strip).insert((
                Name::new("PanelStrip"),
                Node {
                    flex_direction: FlexDirection::Row,
                    padding: SLOT_PADDING,
                    display: Display::None,
                    ..default()
                },
                ChildOf(panel),
            ));
            commands.spawn((Divider::horizontal(), ChildOf(panel)));

            // The body is the one child of the scroll area's contents, so the app's
            // children lay out in an ordinary column.
            commands.entity(body).insert((
                Name::new("PanelBody"),
                Node {
                    flex_direction: FlexDirection::Column,
                    width: Val::Percent(100.0),
                    flex_shrink: 0.0,
                    ..default()
                },
            ));
            commands.entity(scroll_area).insert((
                ScrollArea::bundle(
                    move |s: &mut ChildSpawner| {
                        let contents = s.target_entity();
                        s.world_mut().entity_mut(body).insert(ChildOf(contents));
                    },
                    true,
                    false,
                ),
                ChildOf(panel),
            ));

            commands
                .entity(footer_divider)
                .insert((Divider::horizontal(), ChildOf(panel)));
            commands.entity(footer).insert((
                Name::new("PanelFooter"),
                Node {
                    flex_direction: FlexDirection::Row,
                    padding: SLOT_PADDING,
                    display: Display::None,
                    ..default()
                },
                ChildOf(panel),
            ));

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
        world.commands().run_system_cached_with(update_panel, panel);
    }
}

/// Applies a [`Panel`]'s title, subtitle, close icon and maximum body height to its parts.
fn update_panel(
    panel: In<Entity>,
    panels: Query<(&Panel, &PanelRefs)>,
    labels: Query<&Text, With<Label>>,
    buttons: Query<&IconButton>,
    mut nodes: Query<&mut Node>,
    mut commands: Commands,
) {
    let Ok((panel, refs)) = panels.get(panel.0) else {
        return;
    };
    if labels.get(refs.title).map(|t| t.0.as_str()) != Ok(panel.title.as_str()) {
        commands
            .entity(refs.title)
            .insert(Label::title(panel.title.clone()));
    }
    let subtitle = panel.subtitle.as_deref().unwrap_or("");
    if labels.get(refs.subtitle).map(|t| t.0.as_str()) != Ok(subtitle) {
        commands
            .entity(refs.subtitle)
            .insert(Label::secondary(subtitle));
    }
    set_display(&mut nodes, refs.subtitle, panel.subtitle.is_some());

    if let Some(icon) = &panel.close_icon
        && buttons.get(refs.close).map(|b| &b.icon) != Ok(icon)
    {
        commands.entity(refs.close).insert(IconButton::new(
            icon.clone(),
            "Close",
            IconButtonSize::Small,
        ));
    }
    set_display(&mut nodes, refs.close, panel.close_icon.is_some());

    if let Ok(mut node) = nodes.get_mut(refs.scroll_area) {
        let max_height = Val::Px(panel.max_body_height);
        if node.max_height != max_height {
            node.max_height = max_height;
        }
    }
}

fn set_display(nodes: &mut Query<&mut Node>, entity: Entity, shown: bool) {
    let display = if shown { Display::Flex } else { Display::None };
    if let Ok(mut node) = nodes.get_mut(entity)
        && node.display != display
    {
        node.display = display;
    }
}

/// Shows the strip and the footer of each panel while they have children.
fn show_slots_with_content(
    panels: Query<&PanelRefs>,
    children: Query<(), With<Children>>,
    mut nodes: Query<&mut Node>,
) {
    for refs in &panels {
        set_display(&mut nodes, refs.strip, children.contains(refs.strip));
        let footer = children.contains(refs.footer);
        set_display(&mut nodes, refs.footer, footer);
        set_display(&mut nodes, refs.footer_divider, footer);
    }
}
