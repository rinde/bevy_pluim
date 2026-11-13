//! `Link`: a clickable text that is disabled by [`InteractionDisabled`].

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::{InteractionDisabled, Pressed};
use bevy::ui_widgets::Button;
use itertools::Either;

use crate::focus::{AccessibilitySync, TabStop, sync_label, sync_role};
use crate::theme::{Theme, ThemeColors};
use crate::{MouseEvent, MouseKind};

/// Registers the systems and observers of [`Link`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct LinkPlugin;

impl Plugin for LinkPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_link)
                .run_if(resource_changed::<Theme>),
        )
        .add_systems(Update, restyle_on_disabled_change)
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync));
    }
}

/// Clickable text. Whether it reacts is up to [`InteractionDisabled`].
///
/// It is a [`bevy::ui_widgets::Button`] that triggers
/// [`Activate`](bevy::ui_widgets::Activate) when it is used. Insert
/// [`InteractionDisabled`] to disable it and remove it to enable it again, as for the other
/// buttons: a disabled link keeps its `Button`, shows plain text in the primary text colour
/// (the current crumb of a trail, say), does not react to the pointer, never activates from
/// the pointer or keyboard and is skipped by Tab. The link restyles when the marker is added
/// or removed, also under the pointer.
///
/// To assistive technology it is a link ([`Role::Link`]), not a button.
///
/// Insert a `Link` again to change the label of a live link: its text is updated in place, so
/// the text on screen always equals the accessible label.
///
/// The component is immutable: build or clone a value, edit it outside the world and insert it.
/// It cannot be queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::link::Link;
/// fn edit(mut links: Query<&mut Link>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert = Self::on_insert)]
#[require(BoxShadow::default(),
    Node {
        align_self: AlignSelf::Center,
        justify_self: JustifySelf::Center,
        margin: UiRect::all(Val::Px(4.0)),
        padding: UiRect::all(Val::Px(4.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        ..default()
    },
    Name::new("Link"),
    Button,
    Hovered,
    TabStop,
)]
pub struct Link {
    label: String,
}

#[derive(Component, Clone, Debug, Copy)]
struct LinkText;

impl Link {
    /// A link showing `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    /// Spawns the text and registers the hover observer once; on every later insert sets the
    /// text of the existing child. Runs as one queued command so two inserts before a flush
    /// cannot both spawn a text.
    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        let Some(link) = world.get::<Self>(entity) else {
            return;
        };
        let label = link.label.clone();
        let mut commands = world.commands();
        commands.queue(move |world: &mut World| {
            let existing = world.get::<Children>(entity).and_then(|children| {
                children
                    .iter()
                    .find(|child| world.get::<LinkText>(*child).is_some())
            });
            if let Some(text_id) = existing {
                if let Some(mut text) = world.get_mut::<Text>(text_id)
                    && text.0 != label
                {
                    text.0 = label;
                }
            } else if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                entity_mut.observe(handle_hover);
                world.spawn((
                    Text::new(label),
                    LinkText,
                    Pickable::IGNORE,
                    ChildOf(entity),
                ));
            }
        });
        commands.run_system_cached_with(theme_link, Some(entity));
    }
}

/// Restyles a [`Link`] whose [`InteractionDisabled`] was just added or removed.
fn restyle_on_disabled_change(
    added: Query<Entity, (With<Link>, Added<InteractionDisabled>)>,
    mut removed: RemovedComponents<InteractionDisabled>,
    links: Query<(), With<Link>>,
    mut commands: Commands,
) {
    for entity in added
        .iter()
        .chain(removed.read().filter(|e| links.contains(*e)))
    {
        commands.run_system_cached_with(theme_link, Some(entity));
    }
}

/// Keeps each link a link to assistive technology. Bevy makes a node a button again when its
/// `Button` changes, so this runs after it.
///
/// It looks only at links whose [`AccessibilityNode`] changed (every reset of Bevy's, the
/// `label_changed` and `image_changed` systems and the bounds sync, writes the node through
/// `DerefMut` in `UiSystems::PostLayout`, before this runs) or whose [`Link`] changed.
/// `sync_role` writes only when the role differs, so this never marks the node changed itself.
fn sync_accessibility(
    mut links: Query<
        (&Link, &mut AccessibilityNode),
        Or<(Changed<AccessibilityNode>, Changed<Link>)>,
    >,
) {
    for (link, mut node) in &mut links {
        sync_role(&mut node, Role::Link);
        sync_label(&mut node, &link.label);
    }
}

/// What a [`Link`] is doing, which decides its colours.
#[derive(Clone, Copy)]
struct LinkLook {
    disabled: bool,
    pressed: bool,
    hovered: bool,
}

/// The background and the text colour of a [`Link`], from its state.
const fn colors_for(colors: &ThemeColors, look: LinkLook) -> (Color, Color) {
    let LinkLook {
        disabled,
        pressed,
        hovered,
    } = look;
    if disabled {
        (colors.background_primary, colors.text_primary)
    } else if pressed {
        (colors.background_secondary, colors.link_selected)
    } else if hovered {
        (colors.background_secondary, colors.link)
    } else {
        (colors.background_primary, colors.link)
    }
}

fn handle_hover(
    trigger: On<MouseEvent>,
    mut links: Query<(&mut BackgroundColor, &Children), Without<InteractionDisabled>>,
    mut link_texts: Query<&mut TextColor, With<LinkText>>,
    theme: Res<Theme>,
) {
    // A click and a drag say nothing new about the pointer being on the link: the over,
    // press, release and out events around them set the colours. A drag in particular can
    // arrive after the pointer left the link, so it must not put it back under the pointer.
    let kind = trigger.event().kind;
    if matches!(kind, MouseKind::Click(_) | MouseKind::Drag) {
        return;
    }
    // The pointer is on the link for every event but `Out`, a release included.
    let (bg_color, link_color) = colors_for(
        theme.colors(),
        LinkLook {
            disabled: false,
            pressed: matches!(kind, MouseKind::Pressed),
            hovered: !matches!(kind, MouseKind::Out),
        },
    );
    let Ok((mut link_bg, children)) = links.get_mut(trigger.event().event_target()) else {
        return;
    };
    *link_bg = BackgroundColor(bg_color);

    let Some(text_id) = children.iter().find(|child| link_texts.contains(*child)) else {
        return;
    };
    let Ok(mut text_color) = link_texts.get_mut(text_id) else {
        return;
    };
    text_color.0 = link_color;
}

fn theme_link(
    link_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut links: Query<
        (
            &mut BackgroundColor,
            &Children,
            Has<InteractionDisabled>,
            Has<Pressed>,
            &Hovered,
        ),
        With<Link>,
    >,
    mut link_texts: Query<(&mut TextColor, &mut TextFont), With<LinkText>>,
) {
    let it = if let Some(link_id) = link_id.as_ref() {
        Either::Left(links.get_mut(*link_id).into_iter())
    } else {
        Either::Right(links.iter_mut())
    };
    for (mut link_bg, children, interaction_disabled, pressed, hovered) in it {
        let (bg_color, link_color) = colors_for(
            theme.colors(),
            LinkLook {
                disabled: interaction_disabled,
                pressed,
                hovered: hovered.get(),
            },
        );
        link_bg.0 = bg_color;
        let Some(text_id) = children.iter().find(|child| link_texts.contains(*child)) else {
            continue;
        };
        let Ok((mut text_color, mut text_font)) = link_texts.get_mut(text_id) else {
            continue;
        };
        let face = theme.body_font.clone().into();
        if text_font.font != face {
            text_font.font = face;
        }
        let size = Theme::font_size(theme.font_size_body);
        if text_font.font_size != size {
            text_font.font_size = size;
        }
        text_color.0 = link_color;
    }
}
