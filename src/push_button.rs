use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::Changed;
use bevy::ecs::query::With;
use bevy::ecs::query::Without;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::text::TextColor;
use bevy::text::TextFont;
use bevy::ui::BackgroundColor;
use bevy::ui::BorderColor;
use bevy::ui::BorderRadius;
use bevy::ui::BoxShadow;
use bevy::ui::Interaction;
use bevy::ui::Node;
use bevy::ui::UiRect;
use bevy::ui::widget::Button;
use bevy::ui::widget::Text;
use bevy::utils::default;

use crate::WidgetId;
use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;

#[derive(Debug, Copy, Clone)]
pub struct PushButtonPlugin;

impl Plugin for PushButtonPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, on_hover)
            .add_observer(theme_push_button)
            .add_observer(theme_push_button_label);
    }
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
#[component(on_add = Self::on_add)]
#[require(BoxShadow::default(), Node::default())]
pub struct PushButton(String);

#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct PushButtonLabel;

/// Marker component that disables the default hover effect for `PushButton`s.
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoDefaultHover;

impl PushButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = world.entity(context.entity);
        let button = entity.get::<PushButton>().unwrap();

        let child = (Text::new(&button.0), PushButtonLabel);

        world.commands().entity(context.entity).insert(Button);
        if let Some(&id) = world.entity(context.entity).get::<WidgetId>() {
            world
                .commands()
                .entity(context.entity)
                .with_child((id, child));
        } else {
            world.commands().entity(context.entity).with_child(child);
        }
    }
}

fn on_hover(
    mut query: Query<
        (&Interaction, &mut BackgroundColor, &PushButton),
        (Changed<Interaction>, Without<NoDefaultHover>),
    >,
    theme: Res<Theme>,
) {
    for (interaction, mut background_color, _) in &mut query {
        match interaction {
            Interaction::Hovered => {
                *background_color = BackgroundColor(theme.colors.hovered_item);
            }
            Interaction::None => {
                *background_color = BackgroundColor(theme.colors.background_primary);
            }
            Interaction::Pressed => {
                *background_color = BackgroundColor(theme.colors.selected_item);
            }
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
