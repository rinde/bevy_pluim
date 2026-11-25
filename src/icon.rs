//! `Icon`: an image tinted with a theme color.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::{Theme, ThemeColors};

/// Registers the systems and observers of [`Icon`].
#[derive(Copy, Clone, Debug)]
pub struct IconPlugin;

impl Plugin for IconPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (|| None::<Entity>)
                .pipe(theme_icons)
                .run_if(resource_changed::<Theme>),
        );
    }
}

/// The theme color an [`Icon`] is tinted with.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
pub enum IconColor {
    /// [`ThemeColors::text_primary`].
    #[default]
    TextPrimary,
    /// [`ThemeColors::text_secondary`].
    TextSecondary,
    /// [`ThemeColors::text_tertiary`].
    TextTertiary,
    /// [`ThemeColors::text_disabled`].
    TextDisabled,
    /// [`ThemeColors::accent_text`], the text on the accent fill.
    AccentText,
    /// [`ThemeColors::accent_subtle_text`], the text on the subtle accent.
    AccentSubtleText,
    /// [`ThemeColors::danger_text`].
    DangerText,
    /// [`ThemeColors::warning_text`].
    WarningText,
    /// [`ThemeColors::danger_hover_text`], the text of a dangerous action while hovered.
    DangerHoverText,
}

impl IconColor {
    /// The color this role has in `colors`.
    #[must_use]
    pub const fn color(self, colors: &ThemeColors) -> Color {
        match self {
            Self::TextPrimary => colors.text_primary,
            Self::TextSecondary => colors.text_secondary,
            Self::TextTertiary => colors.text_tertiary,
            Self::TextDisabled => colors.text_disabled,
            Self::AccentText => colors.accent_text,
            Self::AccentSubtleText => colors.accent_subtle_text,
            Self::DangerText => colors.danger_text,
            Self::WarningText => colors.warning_text,
            Self::DangerHoverText => colors.danger_hover_text,
        }
    }
}

/// A square image tinted with a theme color.
///
/// The image must be white on transparent: the tint multiplies it, so a black
/// one stays black.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[require(ImageNode::default(), Node::default(), Name::new("Icon"))]
#[component(on_insert=Self::on_insert)]
pub struct Icon {
    /// The theme color the image is tinted with.
    pub color: IconColor,
}

impl Icon {
    /// An icon of `image`, `size` logical pixels square, tinted with the
    /// primary text color.
    #[must_use]
    pub fn bundle(image: Handle<Image>, size: f32) -> impl Bundle {
        Self::bundle_with_color(image, size, IconColor::TextPrimary)
    }

    /// An icon of `image`, `size` logical pixels square, tinted with `color`.
    #[must_use]
    pub fn bundle_with_color(image: Handle<Image>, size: f32, color: IconColor) -> impl Bundle {
        (
            Self { color },
            ImageNode::new(image),
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                flex_shrink: 0.0,
                ..default()
            },
        )
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        world
            .commands()
            .run_system_cached_with(theme_icons, Some(context.entity));
    }
}

fn theme_icons(
    icon: In<Option<Entity>>,
    theme: Res<Theme>,
    mut icons: Query<(&Icon, &mut ImageNode)>,
) {
    let iter = if let Some(icon) = icon.0 {
        Either::Left(icons.get_mut(icon).into_iter())
    } else {
        Either::Right(icons.iter_mut())
    };

    for (icon, mut image) in iter {
        image.color = icon.color.color(theme.colors());
    }
}
