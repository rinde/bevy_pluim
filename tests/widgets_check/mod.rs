//! `WidgetsCheck`: app checks that read selections and accessibility nodes by what they show.
//! Lives under `tests/` because the library does not depend on the tester.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private test module"
)]

use bevy::a11y::AccessibilityNode;
use bevy::ecs::query::QueryFilter;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::ActiveDescendant;
use bevy_pluim::breadcrumbs::BreadCrumbs;
use bevy_pluim::color_palette::ColorPalette;
use bevy_pluim::combo::{Combo, ComboList};
use bevy_pluim::filter_chips::Chip;
use bevy_pluim::gallery::GalleryCard;
use bevy_pluim::list::List;
use bevy_pluim::select::{Item, Selection, SelectionValue};
use bevy_pluim::tailwind::color_name;
use bevy_pluim::toolbar::Tool;
use bevy_stagecraft::targets::resolve;
use bevy_stagecraft::{AppCheck, CheckResult, Probe, ReflectAppCheck, Target};

/// The checks the widgets' tests make about what the widgets hold.
#[derive(Reflect, Debug, Clone, PartialEq)]
#[reflect(AppCheck)]
pub enum WidgetsCheck {
    /// The items selected in the selection of the widget `of` names are
    /// exactly `items`, in order. Lay the widget out first: wait a frame after setup.
    Selected {
        /// The widget whose selection is read.
        of: Target,
        /// The labels of the selected items, in order.
        items: Vec<String>,
    },
    /// The keyboard focus (`InputFocus`) is on the widget `of` names.
    Focused {
        /// The widget that has the focus.
        of: Target,
    },
    /// The list `of` names has the row `is` names as its `ActiveDescendant`.
    Active {
        /// The list (or palette) that has the active row.
        of: Target,
        /// The row it is.
        is: Target,
    },
    /// The `AccessibilityNode` on `of` has this role, label and value, and every state in
    /// `states`; a `None` field or unlisted state is not checked.
    Accessible {
        /// The widget whose accessibility node is read.
        of: Target,
        /// The role, by its `Debug` name, if it is checked.
        role: Option<String>,
        /// The label, if it is checked.
        label: Option<String>,
        /// The value, if it is checked.
        value: Option<String>,
        /// The states that must hold.
        states: Vec<AccessibleState>,
    },
}

/// A state of an `AccessibilityNode` that [`WidgetsCheck::Accessible`] checks.
// `accesskit` leaves selected, toggled and expanded unset where they do not apply, so
// the negative states require them set and false.
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessibleState {
    /// Selected is set and true.
    Selected,
    /// Selected is set and false.
    NotSelected,
    /// Toggled is set and on.
    Toggled,
    /// Toggled is set and off.
    NotToggled,
    /// Expanded is set and true.
    Expanded,
    /// Expanded is set and false.
    Collapsed,
    /// The disabled flag is set.
    Disabled,
    /// The disabled flag is not set.
    Enabled,
    /// The hidden flag is set.
    Hidden,
    /// The hidden flag is not set.
    Shown,
}

impl AccessibleState {
    /// Whether the state holds on `node`, else what `node` says instead.
    fn check(self, node: &AccessibilityNode) -> Result<(), String> {
        let tristate = |set: Option<bool>, yes: &str, no: &str| match set {
            Some(true) => yes.to_owned(),
            Some(false) => no.to_owned(),
            None => format!("neither {yes} nor {no}"),
        };
        let flag = |set: bool, yes: &str, no: &str| (if set { yes } else { no }).to_owned();
        let toggled = node.toggled().map(|toggled| {
            if toggled == true.into() {
                Some(true)
            } else if toggled == false.into() {
                Some(false)
            } else {
                None
            }
        });
        let (holds, actual) = match self {
            Self::Selected | Self::NotSelected => (
                node.is_selected() == Some(self == Self::Selected),
                tristate(node.is_selected(), "selected", "not selected"),
            ),
            Self::Toggled | Self::NotToggled => (
                toggled == Some(Some(self == Self::Toggled)),
                match toggled {
                    Some(None) => "mixed".to_owned(),
                    other => tristate(other.flatten(), "toggled", "not toggled"),
                },
            ),
            Self::Expanded | Self::Collapsed => (
                node.is_expanded() == Some(self == Self::Expanded),
                tristate(node.is_expanded(), "expanded", "collapsed"),
            ),
            Self::Disabled | Self::Enabled => (
                node.is_disabled() == (self == Self::Disabled),
                flag(node.is_disabled(), "disabled", "enabled"),
            ),
            Self::Hidden | Self::Shown => (
                node.is_hidden() == (self == Self::Hidden),
                flag(node.is_hidden(), "hidden", "shown"),
            ),
        };
        if holds { Ok(()) } else { Err(actual) }
    }
}

/// What differs between the `AccessibilityNode` on `entity` and what was expected.
fn accessible_differences(
    world: &World,
    entity: Entity,
    role: Option<&String>,
    label: Option<&String>,
    value: Option<&String>,
    states: &[AccessibleState],
) -> Result<Vec<String>, String> {
    let node = world
        .get::<AccessibilityNode>(entity)
        .ok_or_else(|| "has no AccessibilityNode".to_owned())?;
    let mut differences = Vec::new();
    if let Some(role) = role {
        let actual = format!("{:?}", node.role());
        if actual != *role {
            differences.push(format!("has role {actual}, not {role}"));
        }
    }
    let mut text = |what: &str, actual: Option<&str>, expected: Option<&String>| {
        let Some(expected) = expected else {
            return;
        };
        match actual {
            Some(actual) if actual == expected => {}
            Some(actual) => differences.push(format!("has {what} {actual:?}, not {expected:?}")),
            None => differences.push(format!("has no {what}, not {expected:?}")),
        }
    };
    text("label", node.label(), label);
    text("value", node.value(), value);
    differences.extend(states.iter().filter_map(|state| {
        state
            .check(node)
            .err()
            .map(|actual| format!("is {actual}, not {state:?}"))
    }));
    Ok(differences)
}

impl AppCheck for WidgetsCheck {
    fn run(&self, probe: &Probe<'_>) -> CheckResult {
        match self {
            Self::Selected { of, items } => {
                let world = probe.world();
                let widget = resolve(world, of)?
                    .widget
                    .ok_or_else(|| format!("{of:?} names a position, not a widget"))?;
                let holder = selection_holder(world, widget)
                    .ok_or_else(|| format!("{of:?} has no selection on it or above it"))?;
                let selected = selected_labels(world, holder)?;
                if selected == *items {
                    Ok(())
                } else {
                    Err(format!("{of:?} has {selected:?} selected, not {items:?}"))
                }
            }
            Self::Active { of, is } => {
                let world = probe.world();
                let list = resolve(world, of)?
                    .widget
                    .ok_or_else(|| format!("{of:?} names a position, not a widget"))?;
                let row = resolve(world, is)?
                    .widget
                    .ok_or_else(|| format!("{is:?} names a position, not a widget"))?;
                match world.get::<ActiveDescendant>(list).and_then(|a| a.0) {
                    Some(active) if active == row => Ok(()),
                    Some(active) => Err(format!("{active} is active in {of:?}, not {is:?}")),
                    None => Err(format!("{of:?} has no active row, not {is:?}")),
                }
            }
            Self::Accessible {
                of,
                role,
                label,
                value,
                states,
            } => {
                let world = probe.world();
                let widget = resolve(world, of)?
                    .widget
                    .ok_or_else(|| format!("{of:?} names a position, not a widget"))?;
                let differences = accessible_differences(
                    world,
                    widget,
                    role.as_ref(),
                    label.as_ref(),
                    value.as_ref(),
                    states,
                )
                .map_err(|missing| format!("{of:?} {missing}"))?;
                if differences.is_empty() {
                    Ok(())
                } else {
                    Err(format!("{of:?} {}", differences.join(", ")))
                }
            }
            Self::Focused { of } => {
                let world = probe.world();
                let widget = resolve(world, of)?
                    .widget
                    .ok_or_else(|| format!("{of:?} names a position, not a widget"))?;
                match world.get_resource::<InputFocus>().and_then(InputFocus::get) {
                    Some(focused) if focused == widget => Ok(()),
                    Some(focused) => Err(format!("{focused} has the focus, not {of:?}")),
                    None => Err(format!("nothing has the focus, not {of:?}")),
                }
            }
        }
    }

    /// One `Selected` per list, combo, trail and palette a single target leads back to.
    fn facts(probe: &Probe<'_>) -> Vec<Self> {
        let world = probe.world();
        let mut facts = Vec::new();
        facts_of::<List, (Without<ColorPalette>, Without<ComboList>)>(world, &mut facts);
        facts_of::<Combo, ()>(world, &mut facts);
        facts_of::<BreadCrumbs, ()>(world, &mut facts);
        facts_of::<ColorPalette, ()>(world, &mut facts);
        facts
    }
}

/// The first entity with a [`Selection`] from `widget` up to its root.
fn selection_holder(world: &World, widget: Entity) -> Option<Entity> {
    core::iter::successors(Some(widget), |entity| {
        world.get::<ChildOf>(*entity).map(ChildOf::parent)
    })
    .find(|entity| world.get::<Selection>(*entity).is_some())
}

/// The labels of the items selected on `holder`, following a [`Selection::Ref`].
fn selected_labels(world: &World, holder: Entity) -> Result<Vec<String>, String> {
    let Some(selection) = world.get::<Selection>(holder) else {
        return Err(format!("{holder} has no selection"));
    };
    let value: &SelectionValue = match selection {
        Selection::Value(value) => value,
        Selection::Ref(referenced) => world
            .get::<Selection>(*referenced)
            .and_then(Selection::as_selection)
            .ok_or_else(|| {
                format!("the selection of {holder} refers to {referenced}, which holds none")
            })?,
    };
    Ok(value.iter().map(|item| label_of(world, item)).collect())
}

/// What an item shows: its text, label, or the name of its colour.
fn label_of(world: &World, item: Entity) -> String {
    if let Some(Item { data }) = world.get::<Item<String>>(item) {
        return data.clone();
    }
    if let Some(Item { data }) = world.get::<Item<Chip>>(item) {
        return data.label.clone();
    }
    if let Some(Item { data }) = world.get::<Item<Tool>>(item) {
        return data.label.clone();
    }
    if let Some(Item { data }) = world.get::<Item<GalleryCard>>(item) {
        return data.label.clone();
    }
    if let Some(Item { data }) = world.get::<Item<Color>>(item) {
        return color_name(*data);
    }
    format!("an item without a label ({item})")
}

/// Adds a `Selected` for each widget with a `W` passing `F` that a target leads back to.
fn facts_of<W: Component + TypePath, F: QueryFilter + 'static>(
    world: &World,
    facts: &mut Vec<WidgetsCheck>,
) {
    let Some(all) = world.try_query_filtered::<Entity, With<W>>() else {
        return;
    };
    let count = all.iter_manual(world).count();
    let Some(query) = world.try_query_filtered::<Entity, (With<W>, With<Selection>, F)>() else {
        return;
    };
    let mut found: Vec<(usize, WidgetsCheck)> = query
        .iter_manual(world)
        .filter_map(|widget| {
            let (place, of) = target_for::<W>(world, widget, count)?;
            let items = selected_labels(world, widget).ok()?;
            Some((place, WidgetsCheck::Selected { of, items }))
        })
        .collect();
    // Entity order is not spawn order; the place among the widgets of the type orders them.
    found.sort_by_key(|(place, _)| *place);
    facts.extend(found.into_iter().map(|(_, fact)| fact));
}

/// A target that resolves to `widget`, with the place it names; `None` when none does.
fn target_for<W: Component + TypePath>(
    world: &World,
    widget: Entity,
    count: usize,
) -> Option<(usize, Target)> {
    let leads_back = |target: &Target| {
        resolve(world, target)
            .ok()
            .and_then(|resolved| resolved.widget)
            == Some(widget)
    };
    let alone = Target::component::<W>();
    if leads_back(&alone) {
        return Some((0, alone));
    }
    (0..count)
        .map(|index| (index, Target::nth(index, Target::component::<W>())))
        .find(|(_, target)| leads_back(target))
}
