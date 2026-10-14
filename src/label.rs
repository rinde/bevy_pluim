//! `Label`: themed text in one of several roles (title, regular, secondary, caption, overline).

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use bevy::text::LetterSpacing;
use bevy::ui::widget::Label as UiLabel;
use itertools::Either;

use crate::theme::Theme;

/// Letter spacing of an overline, in logical pixels.
const OVERLINE_LETTER_SPACING: f32 = 0.5;

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
///
/// It is also Bevy's [`Label`](bevy::ui::widget::Label), so it has an accessibility node with
/// the role `Label` and its text as the value, kept in step when a `Label` is inserted again.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[component(on_insert=Self::on_insert)]
#[require(
    Name::new("Label"),
    UiLabel,
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
    /// A title, for panel titles and headings: display font, the title size.
    Title(String),
    /// Body text.
    Regular(String),
    /// Secondary text, such as descriptions and captions of option bars: secondary text colour.
    Secondary(String),
    /// A caption, such as the faint route line of a row: small size, tertiary text colour.
    Caption(String),
    /// A group heading: the overline size, semibold, secondary text colour, shown in upper case
    /// with 0.5 px of letter spacing.
    Overline(String),
}

impl Label {
    /// A heading with the given text; the same as [`Label::title`].
    pub fn header(text: impl Into<String>) -> Self {
        Self::Title(text.into())
    }

    /// A title with the given text.
    pub fn title(text: impl Into<String>) -> Self {
        Self::Title(text.into())
    }

    /// Secondary text with the given text.
    pub fn secondary(text: impl Into<String>) -> Self {
        Self::Secondary(text.into())
    }

    /// A caption with the given text.
    pub fn caption(text: impl Into<String>) -> Self {
        Self::Caption(text.into())
    }

    /// An overline with the given text; it is shown in upper case.
    pub fn overline(text: impl Into<String>) -> Self {
        Self::Overline(text.into())
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
            Self::Title(text)
            | Self::Regular(text)
            | Self::Secondary(text)
            | Self::Caption(text) => text.clone(),
            Self::Overline(text) => text.to_uppercase(),
        };
        // Inserting Bevy's `Label` again marks it changed, so its system rereads the text.
        world
            .commands()
            .entity(context.entity)
            .insert((Text(text), UiLabel));
        world
            .commands()
            .run_system_cached_with(theme_labels, Some(context.entity));
    }
}

fn theme_labels(
    label_id: In<Option<Entity>>,
    mut labels: Query<(&mut TextColor, &mut TextFont, &mut LetterSpacing, &Label)>,
    theme: Res<Theme>,
) {
    let it = if let Some(button_id) = label_id.as_ref() {
        Either::Left(labels.get_mut(*button_id).into_iter())
    } else {
        Either::Right(labels.iter_mut())
    };

    for (mut text_color, mut text_font, mut letter_spacing, label) in it {
        let (font, font_size, color) = match label {
            Label::Title(_) => (
                &theme.display_font,
                theme.font_size_title,
                theme.colors.text_primary,
            ),
            Label::Regular(_) => (
                &theme.body_font,
                theme.font_size_secondary,
                theme.colors.text_primary,
            ),
            Label::Secondary(_) => (
                &theme.body_font,
                theme.font_size_secondary,
                theme.colors.text_secondary,
            ),
            Label::Caption(_) => (
                &theme.body_font,
                theme.font_size_small,
                theme.colors.text_tertiary,
            ),
            Label::Overline(_) => (
                &theme.body_font,
                theme.font_size_overline,
                theme.colors.text_secondary,
            ),
        };
        let overline = matches!(label, Label::Overline(_));
        *text_color = TextColor(color);
        *text_font = TextFont {
            font: font.clone().into(),
            font_size: FontSize::Px(font_size),
            weight: if overline {
                FontWeight::SEMIBOLD
            } else {
                FontWeight::default()
            },
            ..default()
        };
        *letter_spacing = if overline {
            LetterSpacing::Px(OVERLINE_LETTER_SPACING)
        } else {
            LetterSpacing::default()
        };
    }
}
