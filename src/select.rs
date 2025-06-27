use bevy::ecs::component::Component;
use bevy::ecs::entity::Entity;
use bevy::reflect::Reflect;
use smallvec::SmallVec;

#[derive(Component, Debug, Clone, Default)]
#[component(immutable)]
pub struct Selection {
    pub items: SmallVec<[Entity; 4]>,
    pub multi: bool,
}

impl Selection {
    #[must_use]
    pub fn is_selected(&self, item: Entity) -> bool {
        self.items.contains(&item)
    }

    #[must_use]
    pub fn with_selected(&self, item: Entity) -> Option<Self> {
        (!self.is_selected(item)).then(|| {
            let mut clone = self.clone();
            if self.multi {
                clone.items.push(item);
            } else {
                clone.items.clear();
                clone.items.push(item);
            }
            clone
        })
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = Items)]
// #[component(immutable)]
pub struct ItemOf(pub Entity);

#[derive(Component, Debug, Default, Clone, Reflect)]
#[relationship_target(relationship = ItemOf, linked_spawn)]
#[require(Selection::default())]
pub struct Items(Vec<Entity>);

impl<'a> IntoIterator for &'a Items {
    type Item = Entity;

    type IntoIter = std::iter::Copied<std::slice::Iter<'a, Entity>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter().copied()
    }
}

impl Items {
    pub const fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Component, Debug, Clone)]
pub struct Item {
    pub label: String,
}

impl Item {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl<T: Into<String>> From<T> for Item {
    fn from(value: T) -> Self {
        Self {
            label: value.into(),
        }
    }
}
