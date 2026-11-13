//! The sliding highlight behind the selected item of a track: shared by `SegmentedControl`,
//! `TabBar` and `LightDarkToggle`, which each have an indicator component of their own.

#![expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private, while its items keep pub(crate) to say how far they reach (pub would trip unreachable_pub), as in focus.rs"
)]

use bevy::prelude::*;
use bevy::ui::{ComputedNode, Selected, UiGlobalTransform};
use itertools::Either;
use std::collections::HashMap;

use crate::theme::Theme;

/// How fast an indicator catches up with the selected item, per second: it covers all but about
/// 1 % of the way in a quarter of a second.
const INDICATOR_RATE: f32 = 18.0;
/// The distance in logical pixels under which an indicator snaps to its target.
const INDICATOR_SNAP: f32 = 0.5;

/// The component of an indicator: it belongs to one track.
pub(crate) trait Indicator: Component + Sized {
    /// The indicator of `owner`.
    fn of(owner: Entity) -> Self;
    /// The track this indicator belongs to.
    fn owner(&self) -> Entity;
    /// The colour of the indicator in `theme`.
    fn color(theme: &Theme) -> Color {
        theme.colors().segmented_selected
    }
}

/// Gives every track `T` without one its indicator `I`, drawn behind the items.
///
/// Only tracks that may lack an indicator are looked at, and each by its own children: a track
/// that is new or inserted again (`Changed<T>`), and, when an indicator was removed or despawned
/// (also with its track), every track once, because a removed component no longer says whose it
/// was. A quiet frame does nothing, so the cost no longer grows with tracks times indicators.
///
/// An indicator that was reparented away from its track without being removed is a stray: the
/// track is put back as its parent instead of getting a second indicator. The strays are looked up
/// by owner once per pass, and only when a candidate has no indicator among its children.
pub(crate) fn spawn<I: Indicator, T: Component>(
    changed: Query<Entity, Changed<T>>,
    tracks: Query<Entity, With<T>>,
    children: Query<&Children>,
    indicators: Query<(Entity, &I)>,
    mut removed: RemovedComponents<I>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    let any_removed = removed.read().count() > 0;
    let mut strays: Option<HashMap<Entity, Entity>> = None;
    let candidates = if any_removed {
        Either::Left(tracks.iter())
    } else {
        Either::Right(changed.iter())
    };
    for track in candidates {
        let has_indicator = children.get(track).is_ok_and(|kids| {
            kids.iter()
                .any(|kid| indicators.get(kid).is_ok_and(|(_, i)| i.owner() == track))
        });
        if has_indicator {
            continue;
        }
        let strays = strays
            .get_or_insert_with(|| indicators.iter().map(|(kid, i)| (i.owner(), kid)).collect());
        if let Some(&stray) = strays.get(&track) {
            commands.entity(track).add_child(stray);
            continue;
        }
        commands.entity(track).with_child((
            I::of(track),
            Node {
                position_type: PositionType::Absolute,
                display: Display::None,
                border_radius: BorderRadius::all(theme.control_radius),
                ..default()
            },
            BackgroundColor(I::color(&theme)),
            ZIndex(-1),
            Pickable::IGNORE,
        ));
    }
}

/// Moves each indicator `I` towards the selected item `S` of its track `T`, snapping the first
/// time it is placed. With no selected item the indicator is hidden.
pub(crate) fn animate<I: Indicator, T: Component, S: Component>(
    time: Res<Time>,
    tracks: Query<(&ComputedNode, &UiGlobalTransform, &Children), With<T>>,
    selected: Query<(&ComputedNode, &UiGlobalTransform), (With<S>, With<Selected>)>,
    mut indicators: Query<(&I, &mut Node)>,
) {
    let blend = 1.0 - (-time.delta_secs() * INDICATOR_RATE).exp();
    for (indicator, mut node) in &mut indicators {
        let Ok((track, track_at, children)) = tracks.get(indicator.owner()) else {
            continue;
        };
        let target = children.iter().find_map(|c| selected.get(c).ok());
        let Some((item, item_at)) = target.filter(|(s, _)| s.size().x > 0.0) else {
            if node.display != Display::None {
                node.display = Display::None;
            }
            continue;
        };
        let scale = item.inverse_scale_factor();
        let size = item.size() * scale;
        let origin =
            (item_at.translation - item.size() / 2.0) - (track_at.translation - track.size() / 2.0);
        let goal = [origin.x * scale, origin.y * scale, size.x, size.y];

        let placed = node.display != Display::None;
        let now = [node.left, node.top, node.width, node.height].map(|v| match v {
            Val::Px(px) => px,
            _ => 0.0,
        });
        let next: [f32; 4] = if placed {
            std::array::from_fn(|i| {
                let (from, to) = (
                    now.get(i).copied().unwrap_or(0.0),
                    goal.get(i).copied().unwrap_or(0.0),
                );
                if (to - from).abs() < INDICATOR_SNAP {
                    to
                } else {
                    (to - from).mul_add(blend, from)
                }
            })
        } else {
            goal
        };
        if placed
            && next
                .iter()
                .zip(&now)
                .all(|(a, b)| a.to_bits() == b.to_bits())
        {
            continue;
        }
        let [left, top, width, height] = next.map(Val::Px);
        node.display = Display::Flex;
        node.left = left;
        node.top = top;
        node.width = width;
        node.height = height;
    }
}
