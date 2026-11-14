//! `Kbd`: a keyboard key cap, such as "Esc", shown in running text.

use accesskit::{Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
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
/// colour, in the caption size. For a screen reader it is a label with the key as its value. It reuses those two slots and has none of its own.
///
/// Insert a new `Kbd` to change the key. Empty text keeps the empty box (at its minimum width)
/// and an empty value; it is not hidden.
///
/// The component is immutable: build or clone a value, edit it outside the world and insert it
/// again to update a live widget. It cannot be queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::kbd::Kbd;
/// fn edit(mut widgets: Query<&mut Kbd>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert=Self::on_insert)]
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
        let mut node = AccessNode::new(Role::Label);
        node.set_value(text.as_str());
        world
            .commands()
            .entity(context.entity)
            .insert((Text(text), AccessibilityNode::from(node)));
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
        *border = BorderColor::all(theme.colors().border);
        text_color.set_if_neq(TextColor(theme.colors().text_secondary));
        text_font.set_if_neq(TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(theme.font_size_caption),
            weight: FontWeight::SEMIBOLD,
            ..default()
        });
    }
}
