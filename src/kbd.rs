//! `Kbd`: a keyboard key cap, such as "Esc", shown in running text.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::Theme;

/// Registers the systems and observers of [`Kbd`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct KbdPlugin;

impl Plugin for KbdPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_kbds)
                .run_if(resource_changed::<Theme>),
        );
    }
}

/// A key cap: a small bordered box around the name of a key. Display only.
///
/// The border is the theme's `border` colour, the text its `text_secondary`
/// colour, in the caption size.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[component(on_insert=Self::on_insert)]
#[require(
    Name::new("Kbd"),
    Node {
        min_width: Val::Px(18.0),
        padding: UiRect::axes(Val::Px(5.0), Val::Px(2.0)),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(4.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        align_self: AlignSelf::Center,
        flex_shrink: 0.0,
        ..default()
    },
    BorderColor,
    Pickable {
        should_block_lower: false,
        ..default()
    },
)]
pub struct Kbd(pub String);

impl Kbd {
    /// A key cap showing `key`, such as `"Esc"`.
    pub fn new(key: impl Into<String>) -> Self {
        Self(key.into())
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let Some(this) = world.entity(context.entity).get::<Self>() else {
            return;
        };
        let text = this.0.clone();
        world.commands().entity(context.entity).insert(Text(text));
        world
            .commands()
            .run_system_cached_with(theme_kbds, Some(context.entity));
    }
}

fn theme_kbds(
    kbd: In<Option<Entity>>,
    theme: Res<Theme>,
    mut kbds: Query<(&mut BorderColor, &mut TextColor, &mut TextFont), With<Kbd>>,
) {
    let iter = if let Some(kbd) = kbd.0 {
        Either::Left(kbds.get_mut(kbd).into_iter())
    } else {
        Either::Right(kbds.iter_mut())
    };

    for (mut border, mut text_color, mut text_font) in iter {
        *border = BorderColor::all(theme.colors.border);
        text_color.0 = theme.colors.text_secondary;
        *text_font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: FontSize::Px(theme.font_size_caption),
            weight: FontWeight::SEMIBOLD,
            ..default()
        };
    }
}
