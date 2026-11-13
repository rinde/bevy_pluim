//! The sliding highlight behind the selected item of a track: shared by `SegmentedControl`,
//! `TabBar` and `LightDarkToggle`, which each have an indicator component of their own.

#![expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private, while its items keep pub(crate) to say how far they reach (pub would trip unreachable_pub), as in focus.rs"
)]

use bevy::prelude::*;
use bevy::ui::{ComputedNode, Selected, UiGlobalTransform, UiTransform, Val2};
use itertools::Either;
use std::collections::HashMap;

use crate::theme::Theme;

/// How fast an indicator catches up with the selected item, per second: it covers all but about
/// 1 % of the way in a quarter of a second.
const INDICATOR_RATE: f32 = 18.0;
/// The distance in logical pixels under which an indicator snaps to its target.
const INDICATOR_SNAP: f32 = 0.5;
/// How far in logical pixels the item's box may move before the indicator's `Node` follows it, so
/// float noise from layout does not dirty layout again.
const GOAL_EPSILON: f32 = 0.01;

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
///
/// The `Node` always holds the laid-out box of the selected item (the goal), so layout only runs
/// when the goal moves. The slide is drawn by the indicator's `UiTransform`: its translation is the
/// offset of the visible box's centre from the goal's, its scale the visible size over the goal's,
/// both easing to rest (zero and one). A frame of sliding changes no `Node`, so it dirties no
/// layout.
pub(crate) fn animate<I: Indicator, T: Component, S: Component>(
    time: Res<Time>,
    tracks: Query<(&ComputedNode, &UiGlobalTransform, &Children), With<T>>,
    selected: Query<(&ComputedNode, &UiGlobalTransform), (With<S>, With<Selected>)>,
    mut indicators: Query<(&I, &mut Node, &mut UiTransform)>,
) {
    let blend = 1.0 - (-time.delta_secs() * INDICATOR_RATE).exp();
    for (indicator, mut node, mut transform) in &mut indicators {
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
        let goal = Rect::from_corners(origin * scale, origin * scale + size);

        let placed = node.display != Display::None;
        let px = |v: Val| match v {
            Val::Px(px) => px,
            _ => 0.0,
        };
        let resting = Rect::new(
            px(node.left),
            px(node.top),
            px(node.left) + px(node.width),
            px(node.top) + px(node.height),
        );
        // The visible box now: the node's, moved and scaled by the transform.
        let shown = if placed {
            let offset = Vec2::new(px(transform.translation.x), px(transform.translation.y));
            Rect::from_center_size(resting.center() + offset, resting.size() * transform.scale)
        } else {
            goal
        };

        // The goal moved: the node takes it at once and the transform keeps what is seen where it
        // was.
        let moved = !placed
            || (goal.min - resting.min).abs().max_element() >= GOAL_EPSILON
            || (goal.size() - resting.size()).abs().max_element() >= GOAL_EPSILON;
        if moved {
            node.display = Display::Flex;
            node.left = Val::Px(goal.min.x);
            node.top = Val::Px(goal.min.y);
            node.width = Val::Px(goal.width());
            node.height = Val::Px(goal.height());
        }
        let goal_size = goal.size().max(Vec2::splat(f32::EPSILON));
        let (mut offset, mut ratio) = (shown.center() - goal.center(), shown.size() / goal_size);
        // Ease both towards rest, snapping when near.
        if offset.abs().max_element() < INDICATOR_SNAP
            && (shown.size() - goal.size()).abs().max_element() < INDICATOR_SNAP
        {
            (offset, ratio) = (Vec2::ZERO, Vec2::ONE);
        } else {
            offset *= 1.0 - blend;
            ratio = (ratio - Vec2::ONE).mul_add(Vec2::splat(1.0 - blend), Vec2::ONE);
        }
        let translation = Val2::px(offset.x, offset.y);
        if transform.translation != translation {
            transform.translation = translation;
        }
        if transform.scale != ratio {
            transform.scale = ratio;
        }
    }
}
