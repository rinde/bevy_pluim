use bevy::ecs::component::Component;

#[derive(Component, Debug, Clone, Default)]
#[require(Selection::default())]
pub struct Select {
    pub items: Vec<String>,
}

#[derive(Component, Debug, Clone, Copy, Default)]
pub struct Selection(pub Option<usize>);

impl Select {
    pub fn new(options: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            items: options.into_iter().map(Into::into).collect(),
        }
    }
}
