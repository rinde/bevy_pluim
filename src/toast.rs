//! `Toast`: a transient status message at the bottom center of the window.

use accesskit::{Live, Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;
use itertools::Either;

use crate::card::Card;
use crate::theme::Theme;

const DEFAULT_DURATION_SECS: f32 = 3.0;
/// One day: `Duration` cannot hold longer or infinite values, so durations are capped to it.
const MAX_DURATION_SECS: f32 = 86_400.0;
const MAX_WIDTH: f32 = 480.0;
const GAP: f32 = 8.0;

/// Registers the systems of [`Toast`].
#[derive(Copy, Clone, Debug)]
pub struct ToastPlugin;

impl Plugin for ToastPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ToastPlacement>().add_systems(
            Update,
            (
                (|| None::<Entity>)
                    .pipe(theme_toasts)
                    .run_if(resource_changed::<Theme>),
                (
                    place_toasts.run_if(any_fresh_toast),
                    place_stack.run_if(placement_needs_applying),
                    tick_toasts,
                )
                    .chain(),
            ),
        );
    }
}

/// Where the stack of toasts sits, as a distance from the bottom of the window.
#[derive(Resource, Copy, Clone, Debug, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct ToastPlacement {
    /// The distance from the bottom of the window.
    pub bottom: Val,
}

impl Default for ToastPlacement {
    fn default() -> Self {
        Self { bottom: px(24) }
    }
}

/// A transient message shown bottom center for `duration` seconds, then despawned.
///
/// Spawn it as a child of the UI root: it moves into that parent's [`ToastStack`], and a toast
/// with the same text restarts the one showing. Without a parent it is logged and despawned.
///
/// Insert a new `Toast` on a live entity to change it. The component is immutable, so it cannot
/// be queried mutably:
///
/// ```compile_fail
/// # use bevy::prelude::*;
/// # use bevy_pluim::toast::Toast;
/// fn edit(mut widgets: Query<&mut Toast>) {}
/// # let mut app = App::new();
/// # app.add_systems(Update, edit);
/// # app.update();
/// ```
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert=Self::on_insert)]
#[require(
    Name::new("Toast"),
    Card = Card,
    Node {
        padding: UiRect::axes(px(16), px(10)),
        max_width: px(MAX_WIDTH),
        ..default()
    },
    Text,
    TextLayout,
    ToastTimer,
    Pickable::IGNORE,
)]
pub struct Toast {
    /// The message.
    pub text: String,
    /// How long it shows, in seconds. Negative and NaN mean zero; above one day is capped.
    pub duration: f32,
}

impl Toast {
    /// A toast showing `text` for three seconds.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            duration: DEFAULT_DURATION_SECS,
        }
    }

    /// The same toast shown for `seconds`, see [`Toast::duration`].
    #[must_use]
    pub const fn with_duration(mut self, seconds: f32) -> Self {
        self.duration = seconds;
        self
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let Some(this) = world.entity(context.entity).get::<Self>() else {
            return;
        };
        let text = this.text.clone();
        let timer = ToastTimer::for_duration(this.duration);
        let mut node = AccessNode::new(Role::Status);
        node.set_label(text.as_str());
        node.set_live(Live::Polite);
        world.commands().entity(context.entity).insert((
            Text(text),
            AccessibilityNode::from(node),
            timer,
        ));
        world
            .commands()
            .run_system_cached_with(theme_toasts, Some(context.entity));
    }
}

/// The column the [`Toast`]s of a window are stacked in.
#[derive(Component, Copy, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("ToastStack"),
    Node {
        position_type: PositionType::Absolute,
        left: px(0),
        right: px(0),
        flex_direction: FlexDirection::Column,
        align_items: AlignItems::Center,
        row_gap: px(GAP),
        ..default()
    },
    GlobalZIndex(1100),
    Pickable::IGNORE,
)]
pub struct ToastStack;

#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
struct ToastTimer(Timer);

impl ToastTimer {
    fn for_duration(seconds: f32) -> Self {
        let seconds = if seconds.is_nan() {
            0.0
        } else {
            seconds.clamp(0.0, MAX_DURATION_SECS)
        };
        Self(Timer::from_seconds(seconds, TimerMode::Once))
    }
}

#[derive(Component, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
struct ToastShown;

fn any_fresh_toast(fresh: Query<(), (With<Toast>, Without<ToastShown>)>) -> bool {
    !fresh.is_empty()
}

fn placement_needs_applying(
    placement: Res<ToastPlacement>,
    added: Query<(), Added<ToastStack>>,
) -> bool {
    placement.is_changed() || !added.is_empty()
}

fn place_toasts(
    mut commands: Commands,
    stacks: Query<(Entity, &ChildOf), With<ToastStack>>,
    mut fresh: Query<(Entity, &Toast, &mut ToastTimer, Option<&ChildOf>), Without<ToastShown>>,
    mut shown: Query<(&ChildOf, &Toast, &mut ToastTimer), With<ToastShown>>,
) {
    let mut known: HashMap<Entity, Entity> = stacks
        .iter()
        .map(|(stack, parent)| (parent.parent(), stack))
        .collect();
    let mut kept: Vec<(Entity, String, Entity)> = Vec::new();
    for (entity, toast, mut timer, parent) in &mut fresh {
        let Some(parent) = parent.map(ChildOf::parent) else {
            error!(
                "a Toast ({:?}) has no parent, so it has no ToastStack to join: spawn it as a child of the UI root; despawning it",
                toast.text
            );
            commands.entity(entity).despawn();
            continue;
        };
        let target = *known
            .entry(parent)
            .or_insert_with(|| commands.spawn((ToastStack, ChildOf(parent))).id());
        let existing = shown
            .iter_mut()
            .find(|(of, other, _)| of.parent() == target && other.text == toast.text);
        if let Some((_, _, mut other_timer)) = existing {
            *other_timer = ToastTimer::for_duration(toast.duration);
            commands.entity(entity).despawn();
            continue;
        }
        if let Some((_, _, first)) = kept
            .iter()
            .find(|(s, text, _)| *s == target && *text == toast.text)
        {
            commands
                .entity(*first)
                .insert(ToastTimer::for_duration(toast.duration));
            commands.entity(entity).despawn();
            continue;
        }
        *timer = ToastTimer::for_duration(toast.duration);
        kept.push((target, toast.text.clone(), entity));
        commands
            .entity(entity)
            .insert((ToastShown, ChildOf(target)));
    }
}

fn place_stack(placement: Res<ToastPlacement>, mut stacks: Query<&mut Node, With<ToastStack>>) {
    for mut node in &mut stacks {
        if node.bottom != placement.bottom {
            node.bottom = placement.bottom;
        }
    }
}

fn tick_toasts(
    mut commands: Commands,
    time: Res<Time>,
    mut toasts: Query<(Entity, &mut ToastTimer), With<ToastShown>>,
) {
    for (entity, mut timer) in &mut toasts {
        if timer.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

fn theme_toasts(
    toast: In<Option<Entity>>,
    theme: Res<Theme>,
    mut toasts: Query<(&mut TextColor, &mut TextFont), With<Toast>>,
) {
    let iter = if let Some(toast) = toast.0 {
        Either::Left(toasts.get_mut(toast).into_iter())
    } else {
        Either::Right(toasts.iter_mut())
    };

    for (mut color, mut font) in iter {
        color.set_if_neq(TextColor(theme.colors().text_primary));
        font.set_if_neq(TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(theme.font_size_body),
            ..default()
        });
    }
}
