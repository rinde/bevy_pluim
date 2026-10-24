//! `BreadCrumbs`: a trail of [`Link`]s for a `OneOrMore` selection.
//!
//! Clicking a crumb trims the selection back to it.

use bevy::prelude::*;
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::Activate;

use crate::label::Label;
use crate::link::Link;
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
/// Clicking a crumb trims the selection back to it. The current (last) crumb is a [`Link`] with
/// [`InteractionDisabled`], so it shows plain text and never activates; a crumb that stops being
/// the last is built again without the marker.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
#[require(Name::new("BreadCrumbs"), Node)]
pub struct BreadCrumbs;

/// A crumb of a [`BreadCrumbs`] trail: the selected item it stands for.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
pub struct BreadCrumb(Entity);

fn selection_change(
    q: Query<(), Changed<Selection>>,
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
    // A trail is rebuilt once a run, whether its own selection changed, the one it refers to,
    // or both: two passes would both see the children from before the first and spawn the
    // trail twice.
    for (bread_crumbs_ref, children, list_selection) in lists {
        let is_changed = q.contains(bread_crumbs_ref)
            || matches!(list_selection, Selection::Ref(entity) if q.contains(*entity));
        if is_changed {
            let Some(selection) = selections.get(bread_crumbs_ref) else {
                continue;
            };

            let existing = children
                .map(|c| {
                    c.iter()
                        .filter_map(|e| Some((e, bread_crumb_views.get(e).ok().copied()?)))
                        .collect()
                })
                .unwrap_or(vec![]);

            // Items without text get no crumb. Each crumb keeps its position in the selection,
            // which is the length the selection is trimmed to when the crumb is clicked.
            let selected = selection
                .iter()
                .enumerate()
                .filter(|(_, e)| items.contains(*e))
                .map(|(position, e)| (position, BreadCrumb(e)))
                .collect::<Vec<_>>();

            let first_diff = existing
                .iter()
                .enumerate()
                .find_map(|(i, (_, bc))| {
                    let sel = selected.get(i);
                    sel.is_none_or(|(_, sel)| sel.0 != bc.0).then_some(i)
                })
                .unwrap_or(existing.len())
                .saturating_sub(1);

            for (existing_bc_view, _) in existing.iter().skip(first_diff) {
                commands.entity(*existing_bc_view).despawn();
            }
            for (index, (position, bc)) in selected.iter().enumerate().skip(first_diff) {
                let Ok(item) = items.get(bc.0) else {
                    continue;
                };
                let link = Link::new(item.data.clone());

                // The current crumb is a disabled button: it never activates.
                // A crumb is respawned when it stops being the last, so it is enabled again.
                let current = index + 1 == selected.len();
                if index == 0 {
                    commands.entity(bread_crumbs_ref).with_children(|parent| {
                        parent
                            .spawn((*bc, link))
                            .insert_if(InteractionDisabled, || current)
                            .observe(create_observer(bread_crumbs_ref, position + 1));
                    });
                } else {
                    let separator = Label::regular(" > ");
                    commands.entity(bread_crumbs_ref).with_children(|parent| {
                        parent
                            .spawn((*bc, Node::DEFAULT))
                            .with_children(|sub_parent| {
                                sub_parent.spawn(separator);
                                sub_parent
                                    .spawn(link)
                                    .insert_if(InteractionDisabled, || current)
                                    .observe(create_observer(bread_crumbs_ref, position + 1));
                            });
                    });
                }
            }
        }
    }
}
