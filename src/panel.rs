//! `Panel`: a card with a header, a close button and a body that grows to a
//! maximum height and then scrolls.

use bevy::a11y::AccessibilityNode;
use bevy::asset::HandleTemplate;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::template::{OptionTemplate, Template, TemplateContext};
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::card::Card;
use crate::divider::Divider;
use crate::focus::{clear_label, sync_labelled_by};
use crate::icon_button::{IconButton, IconButtonSize};
use crate::label::Label;
use crate::scroll_area::ScrollArea;
use crate::theme::Theme;

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

/// The default [`Panel::max_body_height()`], in logical pixels.
pub const DEFAULT_MAX_BODY_HEIGHT_PX: f32 = 620.0;

/// A card with a header, a body that scrolls past a maximum height, and a footer.
///
/// The header holds a title, an optional subtitle beside it and an optional
/// close button, with an optional strip under it and a divider. The body grows
/// with its content up to [`Panel::max_body_height()`] and then scrolls; an
/// optional footer sits above a divider.
///
/// The app spawns the contents of the body, the strip and the footer as
/// children of the entities in [`PanelRefs`]; a strip or footer without
/// children is not shown. The close button triggers [`PanelClosed`] on the
/// panel; the panel does not remove itself, the app decides what closing means.
///
/// A `Panel` is an immutable component: insert a new one to change the title, the
/// subtitle, the close button (icon and name) or the maximum height of a live panel.
/// The parts are updated in place, the children the app put in the strip, body and
/// footer stay, and [`PanelClosed`] still arrives once per activation. Nothing watches
/// `Changed<Panel>`.
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::panel::Panel;
///
/// // A panel cannot be edited in place: `&mut Panel` is not a valid query.
/// fn edit(mut panels: Query<&mut Panel>) {}
/// ```
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::panel::Panel;
///
/// fn edit(world: &mut World, panel: Entity) {
///     let _ = world.entity_mut(panel).get_mut::<Panel>();
/// }
/// ```
///
/// To assistive technology it is a group, as a [`Card`] is, labelled with its title.
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert = Self::on_insert)]
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
    /// The accessible name (and tooltip) of the close button, `"Close"` by default;
    /// an app passes its translation. It does not decide whether the button is shown,
    /// [`Panel::close_icon`] does.
    pub close_name: String,
    /// The height, in logical pixels, up to which the body grows before it scrolls;
    /// always finite and above zero (see [`Panel::with_max_body_height`]).
    max_body_height: f32,
}

/// A maximum body height [`Panel`] cannot use: it must be finite and above zero.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PanelHeightError {
    /// The rejected height.
    pub height: f32,
}

impl core::fmt::Display for PanelHeightError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "a panel's maximum body height must be finite and above zero, not {}",
            self.height
        )
    }
}

impl core::error::Error for PanelHeightError {}

/// `height` if a panel can use it as its maximum body height.
fn validate_max_body_height(height: f32) -> core::result::Result<f32, PanelHeightError> {
    if height.is_finite() && height > 0.0 {
        Ok(height)
    } else {
        Err(PanelHeightError { height })
    }
}

/// The entities of a [`Panel`], set when the panel is spawned.
///
/// Read-only: only the widget builds it, and it is an immutable component, so an app
/// reads the entities through the getters and adds its content as their children.
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::panel::PanelRefs;
///
/// // The fields are private: use `PanelRefs::body()`.
/// fn body(refs: &PanelRefs) -> Entity {
///     refs.body
/// }
/// ```
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::panel::PanelRefs;
///
/// fn edit(mut refs: Query<&mut PanelRefs>) {}
/// ```
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
#[component(immutable)]
pub struct PanelRefs {
    header: Entity,
    title: Entity,
    subtitle: Entity,
    close: Entity,
    strip: Entity,
    scroll_area: Entity,
    body: Entity,
    footer: Entity,
    footer_divider: Entity,
}

impl PanelRefs {
    /// The header row.
    #[must_use]
    pub const fn header(&self) -> Entity {
        self.header
    }

    /// The title [`Label`].
    #[must_use]
    pub const fn title(&self) -> Entity {
        self.title
    }

    /// The subtitle [`Label`], not shown while the panel has no subtitle.
    #[must_use]
    pub const fn subtitle(&self) -> Entity {
        self.subtitle
    }

    /// The close [`IconButton`], not shown while the panel has no close icon.
    #[must_use]
    pub const fn close(&self) -> Entity {
        self.close
    }

    /// The strip under the header: spawn its contents as its children.
    #[must_use]
    pub const fn strip(&self) -> Entity {
        self.strip
    }

    /// The [`ScrollArea`] around the body.
    #[must_use]
    pub const fn scroll_area(&self) -> Entity {
        self.scroll_area
    }

    /// The body: spawn its contents as its children.
    #[must_use]
    pub const fn body(&self) -> Entity {
        self.body
    }

    /// The footer: spawn its contents as its children.
    #[must_use]
    pub const fn footer(&self) -> Entity {
        self.footer
    }
}

/// Triggered on a [`Panel`] when its close button is activated.
#[derive(EntityEvent, Debug, Clone, Copy)]
pub struct PanelClosed {
    /// The panel to close.
    pub entity: Entity,
}

/// What `bsn!` patches when a scene writes `Panel { .. }`: a [`Panel`] whose fields
/// not written are those of [`Panel::new`] with an empty title, and whose close icon
/// is a [`HandleTemplate`].
pub struct PanelTemplate {
    /// The title.
    pub title: String,
    /// The subtitle.
    pub subtitle: Option<String>,
    /// The icon of the close button; none, no close button.
    pub close_icon: OptionTemplate<HandleTemplate<Image>>,
    /// The accessible name of the close button.
    pub close_name: String,
    /// The height up to which the body grows before it scrolls.
    pub max_body_height: f32,
}

impl core::fmt::Debug for PanelTemplate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PanelTemplate").finish_non_exhaustive()
    }
}

impl Default for PanelTemplate {
    fn default() -> Self {
        let panel = Panel::new("");
        Self {
            title: panel.title,
            subtitle: panel.subtitle,
            close_icon: OptionTemplate::None,
            close_name: panel.close_name,
            max_body_height: DEFAULT_MAX_BODY_HEIGHT_PX,
        }
    }
}

impl Template for PanelTemplate {
    type Output = Panel;

    fn build_template(&self, context: &mut TemplateContext) -> Result<Panel> {
        let max_body_height = validate_max_body_height(self.max_body_height)?;
        Ok(Panel {
            title: self.title.clone(),
            subtitle: self.subtitle.clone(),
            close_icon: self.close_icon.build_template(context)?,
            close_name: self.close_name.clone(),
            max_body_height,
        })
    }

    fn clone_template(&self) -> Self {
        Self {
            title: self.title.clone(),
            subtitle: self.subtitle.clone(),
            close_icon: self.close_icon.clone_template(),
            close_name: self.close_name.clone(),
            max_body_height: self.max_body_height,
        }
    }
}

impl FromTemplate for Panel {
    type Template = PanelTemplate;
}

impl Panel {
    /// A panel titled `title` with no subtitle or close button and the default
    /// maximum body height.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            close_icon: None,
            close_name: "Close".to_owned(),
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

    /// This panel whose close button is named `name` for assistive technology.
    #[must_use]
    pub fn with_close_name(mut self, name: impl Into<String>) -> Self {
        self.close_name = name.into();
        self
    }

    /// The height, in logical pixels, up to which the body grows before it scrolls.
    #[must_use]
    pub const fn max_body_height(&self) -> f32 {
        self.max_body_height
    }

    /// This panel whose body grows up to `max_body_height` logical pixels.
    ///
    /// # Errors
    ///
    /// Returns a [`PanelHeightError`] unless the height is finite and above zero (NaN,
    /// infinities, zero and negative heights are rejected; nothing is clamped).
    pub fn with_max_body_height(
        mut self,
        max_body_height: f32,
    ) -> core::result::Result<Self, PanelHeightError> {
        self.max_body_height = validate_max_body_height(max_body_height)?;
        Ok(self)
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
            PanelSection(panel),
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
            PanelSection(panel),
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
        // The title is read from its own text, so the group does not repeat it as a label.
        clear_label(&mut accessibility_node);
        sync_labelled_by(&mut accessibility_node, refs.title);
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
        panel.close_name.clone(),
        IconButtonSize::Small,
    ));
    if let Ok(mut node) = nodes.get_mut(refs.subtitle) {
        node.display = show(panel.subtitle.is_some());
    }
    if let Ok(mut node) = nodes.get_mut(refs.close) {
        node.display = show(panel.close_icon.is_some());
    }
    if let Ok(mut node) = nodes.get_mut(refs.scroll_area) {
        node.max_height = Val::Px(panel.max_body_height());
    }
}

/// Marks the strip and the footer of a panel with the panel they belong to, so a change to
/// their children finds the panel without a scan.
#[derive(Component, Clone, Copy, Debug)]
struct PanelSection(Entity);

/// Shows the strip and the footer (and the divider above it) only while they
/// have children, and keeps the body's bottom margin and maximum height.
///
/// Looks only at the panels a change concerns: a new or re-inserted [`Panel`], a strip or
/// footer whose children changed (or lost their last), and every panel when the [`Theme`]
/// changes. A quiet frame reads no panel.
fn sync_optional_sections(
    theme: Res<Theme>,
    changed_panels: Query<Entity, Or<(Changed<Panel>, Changed<PanelRefs>)>>,
    changed_sections: Query<&PanelSection, Changed<Children>>,
    sections: Query<&PanelSection>,
    mut emptied: RemovedComponents<Children>,
    panels: Query<(Entity, &Panel, &PanelRefs)>,
    children: Query<&Children>,
    mut nodes: Query<&mut Node>,
) {
    let wanted: Vec<Entity> = if theme.is_changed() {
        panels.iter().map(|(entity, ..)| entity).collect()
    } else {
        let mut wanted: Vec<Entity> = changed_panels
            .iter()
            .chain(changed_sections.iter().map(|section| section.0))
            .chain(
                emptied
                    .read()
                    .filter_map(|e| sections.get(e).ok().map(|s| s.0)),
            )
            .collect();
        wanted.sort_unstable();
        wanted.dedup();
        wanted
    };
    for entity in wanted {
        let Ok((_, panel, refs)) = panels.get(entity) else {
            continue;
        };
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
        // A scroll area clips to a rectangle, which cannot follow the panel's rounded corners,
        // so with no footer to hold them the body stops a corner radius short of the bottom edge
        // (#384).
        let has_footer = children.get(refs.footer).is_ok_and(|c| !c.is_empty());
        let margin = if has_footer {
            Val::ZERO
        } else {
            theme.panel_radius
        };
        // The margin counts against the maximum height, which is for the body alone.
        let max_height = Val::Px(
            panel.max_body_height()
                + match margin {
                    Val::Px(px) => px,
                    _ => 0.0,
                },
        );
        if let Ok(mut node) = nodes.get_mut(refs.scroll_area)
            && (node.margin.bottom != margin || node.max_height != max_height)
        {
            node.margin.bottom = margin;
            node.max_height = max_height;
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "tests may abort loudly")]
mod tests {
    use super::*;

    #[test]
    fn a_panel_rejects_a_maximum_body_height_that_is_not_finite_and_above_zero() {
        for height in [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            0.0,
            -0.0,
            -1.0,
            f32::MIN,
        ] {
            let error = Panel::new("").with_max_body_height(height).unwrap_err();
            assert_eq!(error.height.to_bits(), height.to_bits());
            assert!(error.to_string().contains("finite and above zero"));
        }
    }

    #[test]
    fn a_panel_accepts_a_fractional_or_custom_maximum_body_height_and_defaults_to_620() {
        assert!((Panel::new("").max_body_height() - 620.0).abs() < f32::EPSILON);
        let tiny = Panel::new("").with_max_body_height(0.001).unwrap();
        assert!((tiny.max_body_height() - 0.001).abs() < f32::EPSILON);
        let custom = Panel::new("").with_max_body_height(250.5).unwrap();
        assert!((custom.max_body_height() - 250.5).abs() < f32::EPSILON);
    }
}
