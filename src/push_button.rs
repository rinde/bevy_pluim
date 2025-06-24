use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::hierarchy::Children;
use bevy::ecs::query::Changed;
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
use bevy::ui::PositionType;
use bevy::ui::UiRect;
use bevy::ui::Val;
use bevy::ui::widget::Button;
use bevy::ui::widget::Text;
use bevy::utils::default;

use crate::theme::Theme;

#[derive(Debug, Copy, Clone)]
pub struct PushButtonPlugin;

impl Plugin for PushButtonPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, on_hover);
    }
}

#[derive(Component, Clone, Debug, Eq, PartialEq)]
#[component(on_add = Self::on_add)]
pub struct PushButton(String);

impl PushButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self(label.into())
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        if let Some(children) = world.entity(context.entity).get::<Children>() {
            println!("{children:?}");
        }

        let button = world.entity(context.entity).get::<PushButton>().unwrap();

        let theme = world.resource::<Theme>();
        let bundle = (
            Button,
            BackgroundColor(theme.colors.background_primary),
            BorderColor(theme.colors.border),
            BorderRadius::all(theme.corner_radius),
            BoxShadow::from(theme.small_shadow()),
            Node {
                position_type: PositionType::Relative,
                border: UiRect::all(theme.border_thickness),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                ..default()
            },
        );
        let child_bundle = (
            Text::new(&button.0),
            TextColor(theme.colors.text_primary),
            TextFont {
                font_size: 14.0,
                ..default()
            },
        );
        world.commands().entity(context.entity).insert(bundle);

        world
            .commands()
            .entity(context.entity)
            .with_child(child_bundle);
    }
}

fn on_hover(
    mut query: Query<(&Interaction, &mut BackgroundColor, &PushButton), Changed<Interaction>>,
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
