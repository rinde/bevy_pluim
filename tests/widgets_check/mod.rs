//! `WidgetsCheck`: the checks of the widgets' tests that the tester's built-in
//! ones cannot say.
//!
//! A [`Selection`] holds entities, which mean nothing in a script, so
//! `Component(..)` cannot say "item `b` is selected". `Selected` names the
//! items by what they show instead. The tester finds this one
//! `#[reflect(AppCheck)]` enum by itself, so a scenario file writes
//! `App(Selected(of: .., items: [..]))` and a Rust scenario
//! `Check::app(WidgetsCheck::Selected { .. })`.
//!
//! Pull it in with `mod widgets_check;`. It lives under `tests/` because the
//! library does not depend on the tester. Like `widgets_setup`, it is its own
//! module and not part of `common`, so a target that does not use it does not
//! fail `dead_code`.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private test module"
)]

use bevy::ecs::query::QueryFilter;
use bevy::input_focus::InputFocus;
use bevy::prelude::*;
use bevy::ui_widgets::ActiveDescendant;
use rin_bevy_ui::breadcrumbs::BreadCrumbs;
use rin_bevy_ui::color_palette::ColorPalette;
use rin_bevy_ui::combo::{Combo, ComboList};
use rin_bevy_ui::list::List;
use rin_bevy_ui::select::{Item, Selection, SelectionValue};
use rin_bevy_ui::tailwind::{COLORS, TailwindColor};
use rin_bevy_ui_tester::targets::resolve;
use rin_bevy_ui_tester::{AppCheck, CheckResult, Probe, ReflectAppCheck, Target};

/// The checks the widgets' tests make about what the widgets hold.
#[derive(Reflect, Debug, Clone, PartialEq)]
#[reflect(AppCheck)]
pub enum WidgetsCheck {
    /// The items selected in the selection of the widget `of` names are
    /// exactly `items`, in the order they were selected.
    ///
    /// `of` resolves to a widget (a `List`, `Combo`, `BreadCrumbs` or
    /// `ColorPalette`, or anything inside one), and the selection is the
    /// first one on it or above it; a `Selection::Ref` is followed. A text
    /// item is named by its label, a colour item by its Tailwind colour
    /// (`blue-500`) when it is one, else by its `Debug` text. The widget has
    /// to be laid out, so a scenario waits a frame after its setup. A bare
    /// selection entity without a `Node` cannot be named by a `Target`, but
    /// every widget that refers to it can.
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
    /// The list `of` names has the row `is` names as its `ActiveDescendant`, the row the
    /// arrow keys moved to.
    Active {
        /// The list (or palette) that has the active row.
        of: Target,
        /// The row it is.
        is: Target,
    },
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

    /// One `Selected` per list, combo, trail and palette that a single target
    /// leads back to, in that order and, within a kind, in the order a script
    /// reads the UI.
    fn facts(probe: &Probe<'_>) -> Vec<Self> {
        let world = probe.world();
        let mut facts = Vec::new();
        // A palette is a list, and a combo has a list of its own; each is
        // proposed once, as what it is.
        facts_of::<List, (Without<ColorPalette>, Without<ComboList>)>(world, &mut facts);
        facts_of::<Combo, ()>(world, &mut facts);
        facts_of::<BreadCrumbs, ()>(world, &mut facts);
        facts_of::<ColorPalette, ()>(world, &mut facts);
        facts
    }
}

/// The first entity with a [`Selection`] on the way from `widget` up to its
/// root, `widget` included.
fn selection_holder(world: &World, widget: Entity) -> Option<Entity> {
    core::iter::successors(Some(widget), |entity| {
        world.get::<ChildOf>(*entity).map(ChildOf::parent)
    })
    .find(|entity| world.get::<Selection>(*entity).is_some())
}

/// The labels of the items selected by the selection on `holder`, following a
/// [`Selection::Ref`].
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

/// What an item shows: its text, or the name of its colour.
fn label_of(world: &World, item: Entity) -> String {
    if let Some(Item { data }) = world.get::<Item<String>>(item) {
        return data.clone();
    }
    if let Some(Item { data }) = world.get::<Item<Color>>(item) {
        return COLORS
            .iter()
            .find(|tailwind| tailwind.color == *data)
            .map_or_else(|| format!("{data:?}"), tailwind_name);
    }
    format!("an item without a label ({item})")
}

/// The Tailwind name of a colour: `blue-500`.
fn tailwind_name(tailwind: &TailwindColor) -> String {
    let class = format!("{:?}", tailwind.class).to_lowercase();
    let variant = format!("{:?}", tailwind.variant);
    format!("{class}-{}", variant.trim_start_matches('V'))
}

/// Adds a `Selected` for each widget with a `W` (and passing the filter `F`)
/// and a selection that a target leads back to, in the order a script reads
/// the UI.
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
    // Entities are not handed out in the order they were spawned, so the
    // place among the widgets of the type is what orders them.
    found.sort_by_key(|(place, _)| *place);
    facts.extend(found.into_iter().map(|(_, fact)| fact));
}

/// A target that resolves to `widget`, with the place it names: the type
/// when it is the only one (place 0), else its place among the `count`
/// widgets of that type, with or without a selection. `None` when no such
/// target leads back to it (a list inside a combo resolves to the combo).
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
