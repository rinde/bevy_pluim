//! `Toast`: a transient status message at the bottom centre of the window.

use accesskit::{Live, Node as AccessNode, Role};
use bevy::a11y::AccessibilityNode;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::prelude::*;
use itertools::Either;

use crate::card::Card;
use crate::theme::Theme;

/// How long a toast shows by default, in seconds.
const DEFAULT_DURATION_SECS: f32 = 3.0;
/// The longest a toast is timed for, in seconds (one day): longer, infinite and
/// huge durations are capped to it, as `Duration` cannot hold them.
const MAX_DURATION_SECS: f32 = 86_400.0;
/// The widest a toast gets, in pixels; longer text wraps.
const MAX_WIDTH: f32 = 480.0;
/// The gap between stacked toasts, in pixels.
const GAP: f32 = 8.0;

/// Registers the systems and observers of [`Toast`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
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
                (place_toasts, place_stack, tick_toasts).chain(),
            ),
        );
    }
}

/// Where the stack of toasts sits: its distance from the bottom edge of the
/// window, so that the application can keep it above what lies there (a tool
/// dock, say).
#[derive(Resource, Copy, Clone, Debug, PartialEq, Reflect)]
#[reflect(Resource)]
pub struct ToastPlacement {
    /// The distance between the lowest toast and the bottom of the window.
    pub bottom: Val,
}

impl Default for ToastPlacement {
    fn default() -> Self {
        Self { bottom: px(24) }
    }
}

/// A transient message: a [`Card`] with the text, shown bottom centre in the
/// window for `duration` seconds and then despawned.
///
/// Spawn it as a child of the UI root. It moves into a [`ToastStack`], where
/// several at once stack upwards with the newest at the bottom; a new toast
/// with the same text restarts the timer of the one showing instead of
/// stacking. It never takes the pointer and is no tab stop. For a screen
/// reader it is a status (a polite live region) with the text.
#[derive(Component, Clone, Debug, PartialEq, Reflect)]
#[reflect(Component)]
#[component(on_insert=Self::on_insert)]
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
    /// How long it shows, in seconds. Zero, a negative value and NaN all mean
    /// zero: the toast goes after its first tick. A value above one day,
    /// infinity included, is capped to one day (86,400 s).
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

    /// The same toast shown for `seconds`. Zero, a negative value and NaN mean
    /// zero (it goes after its first tick); a value above one day, infinity
    /// included, is capped to one day (86,400 s). Nothing here panics.
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

/// The column the [`Toast`]s of a window are stacked in, bottom centre, created
/// by the first toast as a child of its parent. Its [`Node::bottom`] follows
/// [`ToastPlacement`].
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

/// Time left on a [`Toast`]; the toast is despawned when it runs out.
#[derive(Component, Clone, Debug, Default, Reflect)]
#[reflect(Component)]
struct ToastTimer(Timer);

impl ToastTimer {
    /// The timer for a [`Toast::duration`]: NaN and negative values become 0,
    /// anything above [`MAX_DURATION_SECS`] (infinity too) is capped to it.
    fn for_duration(seconds: f32) -> Self {
        let seconds = if seconds.is_nan() {
            0.0
        } else {
            seconds.clamp(0.0, MAX_DURATION_SECS)
        };
        Self(Timer::from_seconds(seconds, TimerMode::Once))
    }
}

/// A [`Toast`] that has moved into its [`ToastStack`].
#[derive(Component, Copy, Clone, Debug, Reflect)]
#[reflect(Component)]
struct ToastShown;

/// Moves new toasts into the stack, dropping one whose text is already
/// showing and restarting the timer of that one instead.
fn place_toasts(
    mut commands: Commands,
    stacks: Query<Entity, With<ToastStack>>,
    mut fresh: Query<(Entity, &Toast, &mut ToastTimer, Option<&ChildOf>), Without<ToastShown>>,
    mut shown: Query<(&Toast, &mut ToastTimer), With<ToastShown>>,
) {
    let mut stack = stacks.iter().next();
    let mut kept: Vec<String> = Vec::new();
    for (entity, toast, mut timer, parent) in &mut fresh {
        let existing = shown.iter_mut().find(|(other, _)| other.text == toast.text);
        if existing.is_some() || kept.contains(&toast.text) {
            if let Some((_, mut other_timer)) = existing {
                *other_timer = ToastTimer::for_duration(toast.duration);
            }
            commands.entity(entity).despawn();
            continue;
        }
        let target = if let Some(target) = stack {
            target
        } else {
            let Some(parent) = parent else {
                continue;
            };
            let target = commands.spawn((ToastStack, ChildOf(parent.parent()))).id();
            stack = Some(target);
            target
        };
        *timer = ToastTimer::for_duration(toast.duration);
        kept.push(toast.text.clone());
        commands
            .entity(entity)
            .insert((ToastShown, ChildOf(target)));
    }
}

/// Keeps the stack at the distance from the bottom that [`ToastPlacement`] says.
fn place_stack(placement: Res<ToastPlacement>, mut stacks: Query<&mut Node, With<ToastStack>>) {
    for mut node in &mut stacks {
        if node.bottom != placement.bottom {
            node.bottom = placement.bottom;
        }
    }
}

/// Counts the toasts down and despawns those that ran out.
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
        color.0 = theme.colors().text_primary;
        *font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(theme.font_size_body),
            ..default()
        };
    }
}
