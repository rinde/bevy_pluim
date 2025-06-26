use bevy::app::Plugin;
use bevy::color::Color;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::Pickable;
use bevy::picking::events::Out;
use bevy::picking::events::Over;
use bevy::picking::events::Pointer;
use bevy::picking::events::Pressed;
use bevy::picking::events::Released;
use bevy::reflect::Reflect;
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

use crate::WidgetId;
use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;
use crate::theme::ThemeColors;

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
                .observe(observer::<Over>(|colors| colors.hovered_item))
                .observe(observer::<Pressed>(|colors| colors.selected_hovered_item))
                .observe(observer::<Released>(|colors| colors.selected_item))
                .observe(observer::<Out>(|colors| colors.background_primary));
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

/// Creates an observer for a given event type that changes the background color
/// of a widget.
fn observer<E: std::fmt::Debug + Clone + Reflect>(
    color: impl Fn(&ThemeColors) -> Color,
) -> impl Fn(Trigger<Pointer<E>>, Res<Theme>, Query<&mut BackgroundColor>) {
    move |trigger: Trigger<Pointer<E>>,
          theme: Res<Theme>,
          mut query: Query<&mut BackgroundColor>| {
        if let Ok(mut background_color) = query.get_mut(trigger.target) {
            *background_color = BackgroundColor(color(&theme.colors));
        }
    }
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
