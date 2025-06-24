use bevy::ecs::bundle::Bundle;
use bevy::ecs::component::Component;
use bevy::ui::BoxShadow;
use bevy::ui::GlobalZIndex;
use bevy::ui::Node;
use bevy::ui::PositionType;

use crate::theme::Theme;

/// Component that modifies the position and appearance of a node to create a
/// floating effect.
#[derive(Component, Copy, Clone, Debug, Eq, PartialEq)]
pub struct Float;

impl Float {
    pub fn apply(&self, node: &mut Node, theme: &Theme) -> impl Bundle {
        node.position_type = PositionType::Absolute;
        (GlobalZIndex(1000), BoxShadow::from(theme.large_shadow()))
    }
}
