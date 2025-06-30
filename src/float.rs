use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::query::With;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ui::BoxShadow;
use bevy::ui::GlobalZIndex;
use bevy::ui::Node;
use bevy::ui::PositionType;
use bevy::ui::Val;

use crate::theme::Theme;

#[derive(Copy, Clone, Debug)]
pub struct FloatPlugin;

impl Plugin for FloatPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, on_theme_change.run_if(resource_changed::<Theme>));
    }
}

/// Component that modifies the position and appearance of a node to create a
/// floating effect.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
#[require(GlobalZIndex(1000), BoxShadow::default())]
pub struct Float;

fn on_theme_change(theme: Res<Theme>, mut query: Query<(&mut Node, &mut BoxShadow), With<Float>>) {
    for (mut node, mut bs) in &mut query {
        node.position_type = PositionType::Absolute;
        node.width = Val::Percent(100.0);
        *bs = BoxShadow::from(theme.large_shadow());
    }
}
