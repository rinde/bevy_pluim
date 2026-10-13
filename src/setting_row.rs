//! `SettingRow`: a title and description on the left, a slot for a control on the right.

use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::theme::Theme;

/// Registers the systems and observers of [`SettingRow`] and [`SettingRows`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct SettingRowPlugin;

impl Plugin for SettingRowPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_setting_rows)
                    .run_if(resource_changed::<Theme>),
                sync_dividers,
            ),
        );
    }
}

/// One setting: a title (15 px semibold) over a description (secondary text) on the
/// left, and on the right a slot for the control, vertically centred.
///
/// Spawn the control as a child of [`SettingRowRefs::slot_id`]. The row draws a divider
/// along its bottom edge when it is inside a [`SettingRows`] and is not its last row;
/// on its own it has none.
#[derive(Component, Clone, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(on_add=Self::on_add)]
#[require(
    Name::new("SettingRow"),
    Node {
        align_items: AlignItems::Center,
        column_gap: Val::Px(12.0),
        padding: UiRect::vertical(Val::Px(12.0)),
        ..default()
    },
    BorderColor,
)]
pub struct SettingRow {
    /// The title.
    pub title: String,
    /// The description under the title.
    pub description: String,
}

/// The children of a [`SettingRow`].
#[derive(Component, Copy, Clone, Debug, PartialEq, Eq, Reflect)]
#[reflect(Component)]
pub struct SettingRowRefs {
    /// The title text.
    pub title_id: Entity,
    /// The description text.
    pub description_id: Entity,
    /// The node on the right to spawn the control under.
    pub slot_id: Entity,
}

impl SettingRow {
    /// A row with the given title and description.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let row = context.entity;
        let Some(this) = world.entity(row).get::<Self>().cloned() else {
            return;
        };
        let mut commands = world.commands();
        let text_id = commands
            .spawn((
                Name::new("SettingRowText"),
                Node {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    flex_shrink: 1.0,
                    ..default()
                },
                ChildOf(row),
            ))
            .id();
        let pickable = Pickable {
            should_block_lower: false,
            ..default()
        };
        let title_id = commands
            .spawn((
                Name::new("SettingRowTitle"),
                SettingRowTitle,
                Text(this.title),
                pickable,
                ChildOf(text_id),
            ))
            .id();
        let description_id = commands
            .spawn((
                Name::new("SettingRowDescription"),
                SettingRowDescription,
                Text(this.description),
                pickable,
                ChildOf(text_id),
            ))
            .id();
        let slot_id = commands
            .spawn((
                Name::new("SettingRowSlot"),
                Node {
                    align_items: AlignItems::Center,
                    flex_shrink: 0.0,
                    ..default()
                },
                ChildOf(row),
            ))
            .id();
        commands.entity(row).insert(SettingRowRefs {
            title_id,
            description_id,
            slot_id,
        });
        commands.run_system_cached_with(theme_setting_rows, Some(row));
    }
}

#[derive(Component, Copy, Clone, Debug)]
struct SettingRowTitle;

#[derive(Component, Copy, Clone, Debug)]
struct SettingRowDescription;

/// A column of [`SettingRow`]s with a divider between consecutive rows and none after
/// the last.
///
/// Spawn the rows as its children; the container keeps the dividers right as rows come
/// and go.
#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("SettingRows"),
    Node {
        flex_direction: FlexDirection::Column,
        ..default()
    },
)]
pub struct SettingRows;

fn theme_setting_rows(
    row: In<Option<Entity>>,
    theme: Res<Theme>,
    mut rows: Query<(&mut BorderColor, &SettingRowRefs), With<SettingRow>>,
    mut titles: Query<(&mut TextColor, &mut TextFont), With<SettingRowTitle>>,
    mut descriptions: Query<
        (&mut TextColor, &mut TextFont),
        (With<SettingRowDescription>, Without<SettingRowTitle>),
    >,
) {
    let iter = if let Some(row) = row.0 {
        Either::Left(rows.get_mut(row).into_iter())
    } else {
        Either::Right(rows.iter_mut())
    };

    for (mut border, refs) in iter {
        *border = BorderColor::all(theme.colors.divider);
        if let Ok((mut color, mut font)) = titles.get_mut(refs.title_id) {
            color.0 = theme.colors.text_primary;
            *font = TextFont {
                font: theme.body_font.clone().into(),
                font_size: FontSize::Px(theme.font_size_body),
                weight: FontWeight::SEMIBOLD,
                ..default()
            };
        }
        if let Ok((mut color, mut font)) = descriptions.get_mut(refs.description_id) {
            color.0 = theme.colors.text_secondary;
            *font = TextFont {
                font: theme.body_font.clone().into(),
                font_size: FontSize::Px(theme.font_size_chip),
                ..default()
            };
        }
    }
}

/// Gives every row of a container but the last a 1 px bottom border.
fn sync_dividers(
    containers: Query<&Children, (With<SettingRows>, Changed<Children>)>,
    mut rows: Query<&mut Node, With<SettingRow>>,
) {
    for children in &containers {
        let row_ids: Vec<Entity> = children
            .iter()
            .filter(|child| rows.contains(*child))
            .collect();
        let last = row_ids.last().copied();
        for row in row_ids {
            let Ok(mut node) = rows.get_mut(row) else {
                continue;
            };
            let width = if Some(row) == last {
                Val::Px(0.0)
            } else {
                Val::Px(1.0)
            };
            if node.border.bottom != width {
                node.border.bottom = width;
            }
        }
    }
}
