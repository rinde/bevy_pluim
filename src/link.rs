use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::MouseEvent;
use crate::theme::Theme;

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
        .add_observer(on_insert_link_state);
    }
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
#[component(on_add = Self::on_add)]
#[require(BoxShadow::default(),
    Node {
        align_self: AlignSelf::Center,
        justify_self: JustifySelf::Center,
        margin: UiRect::all(Val::Px(4.0)),
        padding: UiRect::all(Val::Px(4.0)),
        ..default()
    },
    BorderRadius::all(Val::Px(4.0)),
    Name::new("Link"),
    LinkState::Enabled,
)]
pub struct Link {
    label: String,
}

#[derive(Component, Clone, Debug, Copy)]
struct LinkText;

#[derive(Component, Clone, Debug, Copy, Eq, PartialEq)]
#[component(immutable)]
pub enum LinkState {
    Enabled,
    Disabled,
}

impl Link {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let Some(link) = world.entity(context.entity).get::<Link>() else {
            return;
        };
        let text = Text::new(&link.label);
        let mut commands = world.commands();
        let mut entity_commands = commands.entity(context.entity);
        entity_commands.with_child((
            text,
            TextFont {
                font_size: 18.0,
                ..default()
            },
            LinkText,
            Pickable::IGNORE,
        ));
        // entity_commands.observe(handle_hover);
        world
            .commands()
            .run_system_cached_with(theme_link, Some(context.entity));
    }
}

fn on_insert_link_state(
    t: Trigger<OnInsert, LinkState>,
    q: Query<&LinkState>,
    mut commands: Commands,
) {
    let Ok(state) = q.get(t.target()) else {
        return;
    };
    match state {
        LinkState::Disabled => {
            commands.entity(t.target()).remove::<(Button, Observer)>();
        }
        LinkState::Enabled => {
            let mut observer = Observer::new(handle_hover);
            observer.watch_entity(t.target());
            commands.entity(t.target()).insert((Button, observer));
        }
    }
}

fn handle_hover(
    trigger: Trigger<MouseEvent>,
    mut links: Query<(&mut BackgroundColor, &Children)>,
    mut link_texts: Query<&mut TextColor, With<LinkText>>,
    theme: Res<Theme>,
) {
    let bg_color = match trigger.event() {
        MouseEvent::Over | MouseEvent::Pressed | MouseEvent::Released => {
            theme.colors.background_secondary
        }
        MouseEvent::Out => theme.colors.background_primary,
    };
    let link_color = match trigger.event() {
        MouseEvent::Over | MouseEvent::Released | MouseEvent::Out => theme.colors.link,
        MouseEvent::Pressed => theme.colors.link_selected,
    };
    let Ok((mut link_bg, children)) = links.get_mut(trigger.target()) else {
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
    mut link_texts: Query<&mut TextColor, With<LinkText>>,
) {
    let it = if let Some(link_id) = link_id.as_ref() {
        Either::Left(links.get_mut(*link_id).into_iter())
    } else {
        Either::Right(links.iter_mut())
    };
    for (mut link_bg, children, state) in it {
        link_bg.0 = theme.colors.background_primary;
        let Some(mut text_color) = children
            .iter()
            .next()
            .and_then(|text_ref| link_texts.get_mut(text_ref).ok())
        else {
            continue;
        };
        match state {
            LinkState::Enabled => text_color.0 = theme.colors.link,
            LinkState::Disabled => text_color.0 = theme.colors.text_primary,
        }
    }
}
