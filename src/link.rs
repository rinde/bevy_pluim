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
        );
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
    Button,

)]
pub struct Link {
    label: String,
}

#[derive(Component, Clone, Debug, Copy)]
struct LinkText;

impl Link {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let link = world.entity(context.entity).get::<Link>().unwrap();
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
        entity_commands.observe(handle_hover);
        world
            .commands()
            .run_system_cached_with(theme_link, Some(context.entity));
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
    let (mut link_bg, children) = links.get_mut(trigger.target()).unwrap();
    *link_bg = BackgroundColor(bg_color);

    let mut text_color = link_texts.get_mut(children.iter().next().unwrap()).unwrap();
    text_color.0 = link_color;
}

fn theme_link(
    link_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut links: Query<(&mut BackgroundColor, &Children), With<Link>>,
    mut link_texts: Query<&mut TextColor, With<LinkText>>,
) {
    let it = if let Some(link_id) = link_id.as_ref() {
        Either::Left([links.get_mut(*link_id).unwrap()].into_iter())
    } else {
        Either::Right(links.iter_mut())
    };
    for (mut link_bg, children) in it {
        link_bg.0 = theme.colors.background_primary;
        let text_ref = children.iter().next().unwrap();
        let mut text_color = link_texts.get_mut(text_ref).unwrap();
        text_color.0 = theme.colors.link;
    }
}
