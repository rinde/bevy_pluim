use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::MouseEvent;
use crate::theme::Theme;

#[derive(Debug, Copy, Clone)]
pub struct PushButtonPlugin;

impl Plugin for PushButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_push_button)
                .run_if(resource_changed::<Theme>),
        );
    }
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
#[component(on_add = Self::on_add)]
#[require(BoxShadow::default(),
    Node {
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    },
    Name::new("PushButton"),
)]
pub struct PushButton {
    label: String,
    icon: Option<String>,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonLabel;

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonRefs {
    pub label_id: Entity,
    pub icon_id: Option<Entity>,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonIcon;

/// Marker component that disables the default hover effect for `PushButton`s.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoDefaultHover;

impl PushButton {
    pub fn new(label: impl Into<String>, icon: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: Some(icon.into()),
        }
    }

    pub fn from_label(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let pb = world.entity(context.entity).get::<PushButton>().unwrap();
        let icon = pb
            .icon
            .as_ref()
            .map(|s| (Text::new(s), PushButtonIcon, Pickable::IGNORE));

        let entity = world.entity(context.entity);
        let button = entity.get::<PushButton>().unwrap();
        let label = (Text::new(&button.label), PushButtonLabel, Pickable::IGNORE);

        let default_hover = !entity.contains::<NoDefaultHover>();

        let mut commands = world.commands();
        let mut entity_commands = commands.entity(context.entity);
        entity_commands.insert(Button);

        if default_hover {
            entity_commands.observe(handle_hover);
        }
        let mut label_id = Entity::PLACEHOLDER;
        let mut icon_id = None;
        world
            .commands()
            .entity(context.entity)
            .with_children(|spawner| {
                label_id = spawner.spawn(label).id();
                if let Some(icon) = icon {
                    icon_id = Some(spawner.spawn(icon).id());
                }
            });
        world
            .commands()
            .entity(context.entity)
            .insert(PushButtonRefs { label_id, icon_id });

        world
            .commands()
            .run_system_cached_with(theme_push_button, Some(context.entity));
    }
}

fn handle_hover(
    trigger: Trigger<MouseEvent>,
    mut query: Query<&mut BackgroundColor>,
    theme: Res<Theme>,
) {
    let color = match trigger.event() {
        MouseEvent::Over => theme.colors.hovered_item,
        MouseEvent::Pressed => theme.colors.selected_hovered_item,
        MouseEvent::Released => theme.colors.selected_item,
        MouseEvent::Out => theme.colors.background_primary,
    };
    *query.get_mut(trigger.target()).unwrap() = BackgroundColor(color);
}

fn theme_push_button(
    button_id: In<Option<Entity>>,
    theme: Res<Theme>,
    mut buttons: Query<
        (
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BorderRadius,
            &mut BoxShadow,
            &mut Node,
            &PushButtonRefs,
        ),
        With<PushButton>,
    >,
    mut labels: Query<
        (&mut TextColor, &mut TextFont),
        (With<PushButtonLabel>, Without<PushButtonIcon>),
    >,
    mut icons: Query<
        (&mut TextColor, &mut TextFont),
        (With<PushButtonIcon>, Without<PushButtonLabel>),
    >,
) {
    let it = if let Some(button_id) = button_id.as_ref() {
        Either::Left([buttons.get_mut(*button_id).unwrap()].into_iter())
    } else {
        Either::Right(buttons.iter_mut())
    };
    for (
        mut background_color,
        mut border_color,
        mut border_radius,
        mut box_shadow,
        mut node,
        refs,
    ) in it
    {
        background_color.0 = theme.colors.background_primary;
        border_color.0 = theme.colors.border;
        *border_radius = BorderRadius::all(theme.corner_radius);
        *box_shadow = theme.small_shadow().into();
        node.border = UiRect::all(theme.border_thickness);
        node.padding = theme.item_padding;

        // label
        let (mut text_color, mut text_font) = labels.get_mut(refs.label_id).unwrap();
        *text_color = TextColor(theme.colors.text_primary);
        *text_font = TextFont {
            font_size: 14.0,
            ..default()
        };

        // icon
        if let Some(icon_id) = refs.icon_id {
            let (mut text_color, mut text_font) = icons.get_mut(icon_id).unwrap();
            *text_color = TextColor(theme.colors.border);
            *text_font = TextFont {
                font_size: 14.0,
                ..default()
            };
        }
    }
}
