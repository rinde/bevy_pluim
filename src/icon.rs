//! `Icon`: an image tinted with a theme colour.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::{Theme, ThemeColors};

/// Registers the systems and observers of [`Icon`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
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

/// The theme colour an [`Icon`] is tinted with.
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
}

impl IconColor {
    /// The colour this role has in `colors`.
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
        }
    }
}

/// A square image tinted with a theme colour, so it follows hover, active and
/// theme changes like the text around it.
///
/// Spawn [`Icon::bundle`] with a `Handle<Image>` the application provides; the
/// library ships no icon set. The image must be white on transparent, because
/// the tint multiplies it: a coloured image is tinted too, and a black one
/// stays black. Turn SVGs into white-on-transparent PNGs. Sizes in the design
/// are 16, 18, 22 and 24 px.
///
/// A widget that owns the icon changes the colour role by inserting the
/// `Icon` again, for instance with [`IconColor::AccentText`] while active.
#[derive(Component, Copy, Clone, Debug, Default, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[require(ImageNode::default(), Node::default(), Name::new("Icon"))]
#[component(on_insert=Self::on_insert)]
pub struct Icon {
    /// The theme colour the image is tinted with.
    pub color: IconColor,
}

impl Icon {
    /// An icon of `image`, `size` logical pixels square, tinted with the
    /// primary text colour.
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
        image.color = icon.color.color(&theme.colors);
    }
}
