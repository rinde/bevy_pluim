//! `Link`: a clickable text whose look follows its [`LinkState`].

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Button;
use itertools::Either;

use crate::focus::{AccessibilitySync, TabStop, sync_role};
use crate::theme::Theme;
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
        .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
        .add_observer(on_insert_link_state);
    }
}

/// Clickable text. Whether it reacts is up to its [`LinkState`].
///
/// It is a [`bevy::ui_widgets::Button`] that triggers
/// [`Activate`](bevy::ui_widgets::Activate) when it is used. A
/// [`LinkState::Disabled`] link keeps its `Button` and has [`InteractionDisabled`]
/// instead, so it never activates and does not react to the pointer.
///
/// To assistive technology it is a link ([`Role::Link`]), not a button.
#[derive(Component, Clone, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
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
    LinkState::Enabled,
    TabStop,
)]
pub struct Link {
    label: String,
}

#[derive(Component, Clone, Debug, Copy)]
struct LinkText;

/// Whether a [`Link`] can be clicked.
#[derive(Component, Clone, Debug, Copy, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub enum LinkState {
    /// The link highlights on hover and can be clicked.
    Enabled,
    /// The link is plain text, for example the current crumb of a trail. It is still a
    /// button, with [`InteractionDisabled`].
    Disabled,
}

impl Link {
    /// A link showing `label`.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let Some(link) = world.entity(context.entity).get::<Self>() else {
            return;
        };
        let text = Text::new(&link.label);
        let mut commands = world.commands();
        let mut entity_commands = commands.entity(context.entity);
        entity_commands.with_child((text, LinkText, Pickable::IGNORE));
        entity_commands.observe(handle_hover);
        world
            .commands()
            .run_system_cached_with(theme_link, Some(context.entity));
    }
}

/// Adds or removes [`InteractionDisabled`] to match the [`LinkState`], and restyles the link.
fn on_insert_link_state(t: On<Insert, LinkState>, q: Query<&LinkState>, mut commands: Commands) {
    let link = t.event().event_target();
    let Ok(state) = q.get(link) else {
        return;
    };
    match state {
        LinkState::Disabled => {
            commands.entity(link).insert(InteractionDisabled);
        }
        LinkState::Enabled => {
            commands.entity(link).remove::<InteractionDisabled>();
        }
    }
    commands.run_system_cached_with(theme_link, Some(link));
}

/// Keeps each link a link to assistive technology. Bevy makes a node a button again when its
/// `Button` changes, so this runs after it.
///
/// It runs every frame rather than on change: Bevy's reset of the role happens when the
/// link's `Button` changes, which the link's `AccessibilityNode` does not show, so there is no
/// cheaper filter. `sync_role` writes only when the role differs, so nothing is marked changed.
fn sync_accessibility(mut links: Query<&mut AccessibilityNode, With<Link>>) {
    for mut node in &mut links {
        sync_role(&mut node, Role::Link);
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
    let bg_color = match trigger.event().kind {
        MouseKind::Over | MouseKind::Pressed | MouseKind::Released => {
            theme.colors.background_secondary
        }
        MouseKind::Out => theme.colors.background_primary,
        MouseKind::Click(_) | MouseKind::Drag(_) => return,
    };
    let link_color = match trigger.event().kind {
        MouseKind::Over | MouseKind::Released | MouseKind::Out => theme.colors.link,
        MouseKind::Pressed => theme.colors.link_selected,
        MouseKind::Click(_) | MouseKind::Drag(_) => return,
    };
    let Ok((mut link_bg, children)) = links.get_mut(trigger.event().event_target()) else {
        return;
    };
    *link_bg = BackgroundColor(bg_color);

    let Some(mut text_color) = children
        .iter()
        .next()
        .and_then(|text_ref| link_texts.get_mut(text_ref).ok())
    else {
        return;
    };
    text_color.0 = link_color;
}

fn theme_link(
    link_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut links: Query<(&mut BackgroundColor, &Children, &LinkState), With<Link>>,
    mut link_texts: Query<(&mut TextColor, &mut TextFont), With<LinkText>>,
) {
    let it = if let Some(link_id) = link_id.as_ref() {
        Either::Left(links.get_mut(*link_id).into_iter())
    } else {
        Either::Right(links.iter_mut())
    };
    for (mut link_bg, children, state) in it {
        link_bg.0 = theme.colors.background_primary;
        let Some((mut text_color, mut text_font)) = children
            .iter()
            .next()
            .and_then(|text_ref| link_texts.get_mut(text_ref).ok())
        else {
            continue;
        };
        text_font.font = theme.body_font.clone().into();
        text_font.font_size = FontSize::Px(theme.font_size_body);
        match state {
            LinkState::Enabled => text_color.0 = theme.colors.link,
            LinkState::Disabled => text_color.0 = theme.colors.text_primary,
        }
    }
}
