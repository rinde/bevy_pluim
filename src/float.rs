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
use bevy::ui::Val;

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

fn on_theme_change(
    _: Trigger<ThemeChangedEvent>,
    theme: Res<Theme>,
    mut query: Query<(&mut Node, &mut BoxShadow), With<Float>>,
) {
    for (mut node, mut bs) in &mut query {
        node.position_type = PositionType::Absolute;
        node.width = Val::Percent(100.0);
        *bs = BoxShadow::from(theme.large_shadow());
    }
}
