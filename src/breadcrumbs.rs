//! `BreadCrumbs`: a trail of [`Link`]s for a `OneOrMore` selection.
//!
//! Clicking a crumb trims the selection back to it.

use bevy::prelude::*;
use bevy::ui_widgets::Activate;

use crate::label::Label;
use crate::link::{Link, LinkState};
use crate::select::{Item, Selection, SelectionParam};

/// Registers the systems and observers of [`BreadCrumbs`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct BreadCrumbsPlugin;

impl Plugin for BreadCrumbsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, selection_change);
    }
}

/// A trail of links for the `OneOrMore` selection on this entity (or the one it refers to).
///
/// Clicking a crumb trims the selection back to it.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
#[require(Name::new("BreadCrumbs"), Node{
    ..default()
})]
pub struct BreadCrumbs;

#[derive(Component, Debug, Clone, Copy)]
struct BreadCrumb(Entity);

fn selection_change(
    q: Query<Entity, Changed<Selection>>,
    selections: SelectionParam,
    lists: Query<(Entity, Option<&Children>, &Selection), With<BreadCrumbs>>,
    items: Query<&Item<String>>,
    bread_crumb_views: Query<&BreadCrumb>,
    mut commands: Commands,
) {
    let create_observer = |selection_ref: Entity, new_length: usize| {
        move |_: On<Activate>, mut selections: SelectionParam| {
            selections.update_selection(selection_ref, |s| s.trim_to_length(new_length));
        }
    };
    for changed_selection_ref in q {
        for (bread_crumbs_ref, children, list_selection) in lists {
            let is_changed = match list_selection {
                Selection::Ref(entity) => {
                    changed_selection_ref == *entity || changed_selection_ref == bread_crumbs_ref
                }
                Selection::Value(_selection) => changed_selection_ref == bread_crumbs_ref,
            };
            if is_changed {
                let Some(selection) = selections.get(changed_selection_ref) else {
                    continue;
                };

                let existing = children
                    .map(|c| {
                        c.iter()
                            .filter_map(|e| Some((e, bread_crumb_views.get(e).ok().copied()?)))
                            .collect()
                    })
                    .unwrap_or(vec![]);

                let selected = selection.iter().map(BreadCrumb).collect::<Vec<_>>();

                let first_diff = existing
                    .iter()
                    .enumerate()
                    .find_map(|(i, (_, bc))| {
                        let sel = selected.get(i);
                        sel.is_none_or(|sel| sel.0 != bc.0).then_some(i)
                    })
                    .unwrap_or(existing.len())
                    .saturating_sub(1);

                for (existing_bc_view, _) in existing.iter().skip(first_diff) {
                    commands.entity(*existing_bc_view).despawn();
                }
                for (index, bc) in selected.iter().skip(first_diff).enumerate() {
                    let Ok(item) = items.get(bc.0) else {
                        continue;
                    };
                    let link = Link::new(item.data.clone());
                    let current_index = index + first_diff;

                    // The current crumb is a disabled button: it never activates.
                    let link = if current_index == selected.len() - 1 {
                        (link, LinkState::Disabled)
                    } else {
                        (link, LinkState::Enabled)
                    };
                    if current_index == 0 {
                        commands.entity(bread_crumbs_ref).with_children(|parent| {
                            parent
                                .spawn((*bc, link))
                                .observe(create_observer(changed_selection_ref, current_index + 1));
                        });
                    } else {
                        let separator = Label::regular(" > ");
                        commands.entity(bread_crumbs_ref).with_children(|parent| {
                            parent
                                .spawn((*bc, Node::DEFAULT))
                                .with_children(|sub_parent| {
                                    sub_parent.spawn(separator);
                                    sub_parent.spawn(link).observe(create_observer(
                                        changed_selection_ref,
                                        current_index + 1,
                                    ));
                                });
                        });
                    }
                }
            }
        }
    }
}
