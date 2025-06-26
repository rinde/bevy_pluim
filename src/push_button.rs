use bevy::app::Plugin;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::Pickable;
use bevy::text::TextColor;
use bevy::text::TextFont;
use bevy::ui::AlignItems;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::BoxShadow;
use bevy::ui::JustifyContent;
use bevy::ui::Node;
use bevy::ui::UiRect;
use bevy::ui::widget::Button;
use bevy::ui::widget::Text;
use bevy::utils::default;

use crate::EntityCommandsExt;
use crate::MouseEvent;
use crate::WidgetId;
use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;

#[derive(Debug, Copy, Clone)]
pub struct PushButtonPlugin;

impl Plugin for PushButtonPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_observer(theme_push_button)
            .add_observer(theme_push_button_label)
            .add_observer(theme_push_button_icon);
    }
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
#[component(on_add = Self::on_add)]
#[require(BoxShadow::default(),
    Node{
        justify_content: JustifyContent::SpaceBetween,
        align_items: AlignItems::Center,
        ..default()
    }
)]
pub struct PushButton {
    label: String,
    icon: Option<String>,
}

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonLabel;

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
        entity_commands.insert(Button); //.observe(observe_click);

        if default_hover {
            entity_commands
                .observe_and_transform_mouse_events()
                .observe(handle_hover);
        }
        if let Some(&id) = world.entity(context.entity).get::<WidgetId>() {
            world
                .commands()
                .entity(context.entity)
                .with_children(|spawner| {
                    spawner.spawn((id, label));
                    if let Some(icon) = icon {
                        spawner.spawn((id, icon));
                    }
                });
        } else {
            world
                .commands()
                .entity(context.entity)
                .with_children(|spawner| {
                    spawner.spawn(label);
                    if let Some(icon) = icon {
                        spawner.spawn(icon);
                    }
                });
        }
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
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut query: Query<
        (
            &mut BackgroundColor,
            &mut BorderColor,
            &mut BorderRadius,
            &mut BoxShadow,
            &mut Node,
        ),
        With<PushButton>,
    >,
) {
    for (mut background_color, mut border_color, mut border_radius, mut box_shadow, mut node) in
        &mut query
    {
        background_color.0 = theme.colors.background_primary;
        border_color.0 = theme.colors.border;
        *border_radius = BorderRadius::all(theme.corner_radius);
        *box_shadow = theme.small_shadow().into();
        node.border = UiRect::all(theme.border_thickness);
        node.padding = theme.item_padding;
    }
}

fn theme_push_button_label(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut query: Query<(&mut TextColor, &mut TextFont), With<PushButtonLabel>>,
) {
    for (mut text_color, mut text_font) in &mut query {
        *text_color = TextColor(theme.colors.text_primary);
        *text_font = TextFont {
            font_size: 14.0,
            ..default()
        };
    }
}

fn theme_push_button_icon(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut query: Query<(&mut TextColor, &mut TextFont), With<PushButtonIcon>>,
) {
    for (mut text_color, mut text_font) in &mut query {
        *text_color = TextColor(theme.colors.border);
        *text_font = TextFont {
            font_size: 14.0,
            ..default()
        };
    }
}
