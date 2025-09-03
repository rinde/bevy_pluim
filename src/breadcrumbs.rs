use bevy::prelude::*;

use crate::label::Label;
use crate::link::Link;
use crate::select::{Item, Selection, SelectionParam};

#[derive(Copy, Clone, Debug)]
pub struct BreadCrumbsPlugin;

impl Plugin for BreadCrumbsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, selection_change);
    }
}

#[derive(Component, Debug, Clone, Copy)]
#[require(Name::new("BreadCrumbs"), Node{
    ..default()
})]
pub struct BreadCrumbs;

fn selection_change(
    q: Query<Entity, Changed<Selection>>,
    selections: SelectionParam,
    lists: Query<(Entity, Option<&Children>, &Selection), With<BreadCrumbs>>,
    items: Query<&Item<String>>,
    bread_crumb_views: Query<&BreadCrumb>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (bread_crumbs_ref, children, list_selection) in lists {
            let is_changed = match list_selection {
                Selection::Ref(entity) => {
                    changed_selection_ref == *entity || changed_selection_ref == bread_crumbs_ref
                }
                Selection::Value(_selection) => changed_selection_ref == bread_crumbs_ref,
            };
            if is_changed {
                let selection = selections.get(changed_selection_ref).unwrap_or_else(|| {
                    panic!("No selection found for entity {changed_selection_ref}")
                });

                let existing = children
                    .map(|c| {
                        c.iter()
                            .map(|e| (e, bread_crumb_views.get(e).copied().unwrap()))
                            .collect()
                    })
                    .unwrap_or(vec![]);

                let selected = selection.iter().map(BreadCrumb).collect::<Vec<_>>();

                let first_diff = existing.iter().enumerate().find_map(|(i, (_, bc))| {
                    let sel = selected.get(i);
                    sel.is_none_or(|sel| sel.0 != bc.0).then_some(i)
                });

                if let Some(first_diff) = first_diff {
                    for (existing_bc_view, _) in &existing[first_diff..] {
                        commands.entity(*existing_bc_view).despawn();
                    }
                }
                // TODO avoid double first

                let add_from = first_diff.unwrap_or(existing.len()).saturating_sub(1);
                for (index, bc) in selected[add_from..].iter().enumerate() {
                    let item = items.get(bc.0).unwrap();
                    let link = Link::new(item.data.clone());
                    if index + add_from == 0 {
                        commands.entity(bread_crumbs_ref).with_child((*bc, link));
                    } else if index + add_from == selected.len() - 1 {
                        commands
                            .entity(bread_crumbs_ref)
                            .with_child((*bc, Label::header(format!(" > {}", item.data.clone()))));
                    } else {
                        let separator = Label::regular(" > ");
                        commands.entity(bread_crumbs_ref).with_child((
                            *bc,
                            Node::DEFAULT,
                            children![separator, link],
                        ));
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug, Clone, Copy)]
struct BreadCrumb(Entity);
