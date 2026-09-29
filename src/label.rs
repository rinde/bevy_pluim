//! `Label`: themed text, either a header or regular text.

use bevy::ecs::component::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::Theme;

pub(crate) struct LabelPlugin;

impl Plugin for LabelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            ((|| None::<Entity>)
                .pipe(theme_labels)
                .run_if(resource_changed::<Theme>),),
        );
    }
}

/// Themed text.
#[derive(Component, Clone, Debug)]
#[component(on_insert=Self::on_insert)]
#[require(
    Name::new("Label"),
    Node {
        align_self: AlignSelf::Center,
        ..default()
    },
    Pickable {
        should_block_lower: false,
        ..default()
    },
)]
pub enum Label {
    /// Larger text, for headings.
    Header(String),
    /// Body text.
    Regular(String),
}

impl Label {
    /// A heading with the given text.
    pub fn header(text: impl Into<String>) -> Self {
        Self::Header(text.into())
    }

    /// Body text with the given text.
    pub fn regular(text: impl Into<String>) -> Self {
        Self::Regular(text.into())
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let Some(this) = world.entity(context.entity).get::<Self>() else {
            return;
        };
        let text = match this {
            Self::Header(text) | Self::Regular(text) => text.clone(),
        };
        world.commands().entity(context.entity).insert(Text(text));
        world
            .commands()
            .run_system_cached_with(theme_labels, Some(context.entity));
    }
}

fn theme_labels(
    label_id: In<Option<Entity>>,
    mut labels: Query<(&mut TextColor, &mut TextFont, &Label)>,
    theme: Res<Theme>,
) {
    let it = if let Some(button_id) = label_id.as_ref() {
        Either::Left(labels.get_mut(*button_id).into_iter())
    } else {
        Either::Right(labels.iter_mut())
    };

    for (mut text_color, mut text_font, label) in it {
        let font_size = match label {
            Label::Header(_) => 18.0,
            Label::Regular(_) => 14.0,
        };
        *text_color = TextColor(theme.colors.text_primary);
        *text_font = TextFont {
            font_size,
            ..default()
        };
    }
}
