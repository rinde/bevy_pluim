//! `Link`: a clickable text whose look follows its [`LinkState`].

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
///
/// Insert a `Link` again to change the label of a live link: its text is updated in place, so
/// the text on screen always equals the accessible label.
#[derive(Component, Clone, Debug, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(on_insert = Self::on_insert)]
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

/// Adds or removes [`InteractionDisabled`] to match the [`LinkState`], and restyles the link.
fn on_insert_link_state(t: On<Insert<LinkState>>, q: Query<&LinkState>, mut commands: Commands) {
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
fn sync_accessibility(mut links: Query<(&Link, &mut AccessibilityNode)>) {
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
        &theme.colors,
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
    mut links: Query<
        (
            &mut BackgroundColor,
            &Children,
            &LinkState,
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
    for (mut link_bg, children, state, interaction_disabled, pressed, hovered) in it {
        let (bg_color, link_color) = colors_for(
            &theme.colors,
            LinkLook {
                disabled: interaction_disabled || matches!(state, LinkState::Disabled),
                pressed,
                hovered: hovered.get(),
            },
        );
        link_bg.0 = bg_color;
        let Some((mut text_color, mut text_font)) = children
            .iter()
            .next()
            .and_then(|text_ref| link_texts.get_mut(text_ref).ok())
        else {
            continue;
        };
        text_font.font = theme.body_font.clone().into();
        text_font.font_size = Theme::font_size(theme.font_size_body);
        text_color.0 = link_color;
    }
}
