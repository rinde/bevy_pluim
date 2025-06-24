use bevy::app::App;
use bevy::app::Plugin;
use bevy::ecs::component::Component;
use bevy::ecs::observer::Trigger;
use bevy::ecs::query::With;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ui::BoxShadow;
use bevy::ui::GlobalZIndex;
use bevy::ui::Node;
use bevy::ui::PositionType;

use crate::theme::Theme;
use crate::theme::ThemeChangedEvent;

#[derive(Copy, Clone, Debug)]
pub struct FloatPlugin;

impl Plugin for FloatPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_theme_change);
    }
}

/// Component that modifies the position and appearance of a node to create a
/// floating effect.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
#[require(GlobalZIndex(1000), BoxShadow::default())]
pub struct Float;

impl Float {
    pub const fn apply(&self, node: &mut Node) {
        node.position_type = PositionType::Absolute;
    }
}

fn on_theme_change(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut query: Query<&mut BoxShadow, With<Float>>,
) {
    for mut bs in &mut query {
        *bs = BoxShadow::from(theme.large_shadow());
    }
}
