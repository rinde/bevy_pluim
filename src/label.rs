use bevy::app::App;
use bevy::app::Plugin;
use bevy::app::Update;
use bevy::ecs::component::Component;
use bevy::ecs::component::HookContext;
use bevy::ecs::entity::Entity;
use bevy::ecs::name::Name;
use bevy::ecs::schedule::IntoScheduleConfigs;
use bevy::ecs::schedule::common_conditions::resource_changed;
use bevy::ecs::system::In;
use bevy::ecs::system::IntoSystem;
use bevy::ecs::system::Query;
use bevy::ecs::system::Res;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::Pickable;
use bevy::text::TextColor;
use bevy::text::TextFont;
use bevy::ui::AlignSelf;
use bevy::ui::Node;
use bevy::ui::widget::Text;
use bevy::utils::default;
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
    Header(String),
    Regular(String),
}

impl Label {
    pub fn header(text: impl Into<String>) -> Self {
        Label::Header(text.into())
    }

    pub fn regular(text: impl Into<String>) -> Self {
        Label::Regular(text.into())
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let this = world.entity(context.entity).get::<Self>().unwrap();
        let text = match this {
            Label::Header(text) | Label::Regular(text) => text.clone(),
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
        Either::Left([labels.get_mut(*button_id).unwrap()].into_iter())
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
