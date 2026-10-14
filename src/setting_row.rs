//! `SettingRow`: a title over a description on the left and a slot for a control on the right.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::divider::Divider;
use crate::label::Label;
use crate::theme::Theme;

/// Registers the systems and observers of [`SettingRow`].
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

/// A row of a preferences panel: a title (semibold) over a description (secondary) on the left,
/// and a control on the right.
///
/// The row does not make the control: spawn it as a child of
/// [`SettingRowRefs::control_slot`]. Put the rows in a [`SettingRows`] column, which keeps the
/// divider between consecutive rows and none after the last.
///
/// To assistive technology it is a group ([`Role::Group`]) labelled with its title and
/// described by its description.
#[derive(Component, Clone, Debug, Reflect)]
#[reflect(Component)]
#[component(on_add=Self::on_add)]
#[require(
    Name::new("SettingRow"),
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::Group)),
    Node {
        flex_direction: FlexDirection::Column,
        align_self: AlignSelf::Stretch,
        ..default()
    },
)]
pub struct SettingRow {
    title: String,
    description: String,
}

/// The children of a [`SettingRow`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettingRowRefs {
    /// The title text.
    pub title_id: Entity,
    /// The description text.
    pub description_id: Entity,
    /// The node on the right that the app spawns the control into.
    pub control_slot: Entity,
    /// The divider under the row, hidden while the row is the last of its [`SettingRows`].
    pub divider_id: Entity,
}

/// Marks the title text of a [`SettingRow`].
#[derive(Component, Clone, Copy, Debug, Eq, PartialEq)]
pub struct SettingRowTitle;

/// A column of [`SettingRow`]s; the last row shows no divider.
#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("SettingRows"),
    Node {
        flex_direction: FlexDirection::Column,
        align_self: AlignSelf::Stretch,
        ..default()
    },
)]
pub struct SettingRows;

impl SettingRow {
    /// A row with the given title and description.
    pub fn new(title: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
        }
    }

    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let Some(row) = world.entity(context.entity).get::<Self>() else {
            return;
        };
        let title = row.title.clone();
        let description = row.description.clone();
        if let Some(mut node) = world.get_mut::<AccessibilityNode>(context.entity) {
            node.set_label(title.as_str());
            node.set_description(description.as_str());
        }

        let mut title_id = Entity::PLACEHOLDER;
        let mut description_id = Entity::PLACEHOLDER;
        let mut control_slot = Entity::PLACEHOLDER;
        let mut divider_id = Entity::PLACEHOLDER;
        world
            .commands()
            .entity(context.entity)
            .with_children(|spawner| {
                spawner
                    .spawn((
                        Name::new("SettingRowContent"),
                        Node {
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            column_gap: Val::Px(12.0),
                            padding: UiRect::vertical(Val::Px(12.0)),
                            ..default()
                        },
                    ))
                    .with_children(|content| {
                        content
                            .spawn((
                                Name::new("SettingRowText"),
                                Node {
                                    flex_direction: FlexDirection::Column,
                                    flex_grow: 1.0,
                                    flex_basis: Val::Px(0.0),
                                    row_gap: Val::Px(2.0),
                                    ..default()
                                },
                            ))
                            .with_children(|text| {
                                title_id = text
                                    .spawn((
                                        Text::new(title),
                                        SettingRowTitle,
                                        Node {
                                            align_self: AlignSelf::FlexStart,
                                            ..default()
                                        },
                                        Pickable::IGNORE,
                                    ))
                                    .id();
                                description_id = text
                                    .spawn((
                                        Label::secondary(description),
                                        Node {
                                            align_self: AlignSelf::FlexStart,
                                            ..default()
                                        },
                                    ))
                                    .id();
                            });
                        control_slot = content
                            .spawn((
                                Name::new("SettingRowControlSlot"),
                                Node {
                                    align_items: AlignItems::Center,
                                    flex_shrink: 0.0,
                                    ..default()
                                },
                            ))
                            .id();
                    });
                divider_id = spawner.spawn(Divider::horizontal()).id();
            });
        world
            .commands()
            .entity(context.entity)
            .insert(SettingRowRefs {
                title_id,
                description_id,
                control_slot,
                divider_id,
            });
        world
            .commands()
            .run_system_cached_with(theme_setting_rows, Some(context.entity));
    }
}

fn theme_setting_rows(
    row: In<Option<Entity>>,
    theme: Res<Theme>,
    rows: Query<&SettingRowRefs>,
    mut titles: Query<(&mut TextColor, &mut TextFont), With<SettingRowTitle>>,
) {
    let iter = row.0.map_or_else(
        || Either::Right(rows.iter().map(|refs| refs.title_id)),
        |row| Either::Left(rows.get(row).into_iter().map(|refs| refs.title_id)),
    );
    for title_id in iter {
        let Ok((mut color, mut font)) = titles.get_mut(title_id) else {
            continue;
        };
        *color = TextColor(theme.colors.text_primary);
        *font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: FontSize::Px(theme.font_size_body),
            weight: FontWeight::SEMIBOLD,
            ..default()
        };
    }
}

/// Hides the divider of the last row of each [`SettingRows`] and shows it on the others.
fn sync_dividers(
    containers: Query<&Children, With<SettingRows>>,
    rows: Query<&SettingRowRefs>,
    mut nodes: Query<&mut Node>,
) {
    for children in &containers {
        let rows_here: Vec<&SettingRowRefs> =
            children.iter().filter_map(|c| rows.get(c).ok()).collect();
        let last = rows_here.len().saturating_sub(1);
        for (i, refs) in rows_here.into_iter().enumerate() {
            let Ok(mut node) = nodes.get_mut(refs.divider_id) else {
                continue;
            };
            let display = if i == last {
                Display::None
            } else {
                Display::Flex
            };
            if node.display != display {
                node.display = display;
            }
        }
    }
}
