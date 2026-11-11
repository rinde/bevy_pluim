//! The text page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn labels(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Labels")}
                --
                ~{groups()}
                Children [
                    @variant("Header", bsn! { ~{Label::header("Header")} })
                    --
                    @variant("Title", bsn! { ~{Label::title("Title")} })
                    --
                    @variant("Regular", bsn! { ~{Label::regular("Regular text")} })
                    --
                    @variant("Secondary", bsn! { ~{Label::secondary("Secondary text")} })
                    --
                    @variant("Caption", bsn! { ~{Label::caption("Caption")} })
                    --
                    @variant("Overline", bsn! { ~{Label::overline("Overline")} })
                    --
                    @variant("Divider", bsn! {
                        @{Divider::horizontal_scene()}
                        Node { width: px(200.0) }
                    })
                    --
                    @variant("Key caps", bsn! {
                        ~{row()}
                        Children [
                            ~{Label::regular("Press")}
                            --
                            ~{Kbd::new("Esc")}
                            --
                            ~{Label::regular("to cancel, or")}
                            --
                            ~{Kbd::new("R")}
                            --
                            ~{Label::regular("to rotate")}
                        ]
                    })
                ]
            ]
        },
    );
}

pub(super) fn tags(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    let (neutral, warning, plain) = (icon.clone(), icon.clone(), icon);
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Tags")}
                --
                ~{groups()}
                Children [
                    @variant("Pill: neutral", bsn! { ~{Pill::new("Idle")} })
                    --
                    @variant("Pill: info", bsn! { ~{Pill::new("Running").with_tone(PillTone::Info)} })
                    --
                    @variant("Pill: warning (waiting)", bsn! {
                        ~{Pill::new("Waiting").with_tone(PillTone::Warning)}
                    })
                    --
                    @variant("Pill: problem", bsn! {
                        ~{Pill::new("No path").with_tone(PillTone::Problem)}
                    })
                    --
                    @variant("Badge: neutral count", bsn! {
                        IconButton { icon: {neutral}, name: "Trains", size: IconButtonSize::Large }
                        ~{Badge::count(3)}
                        Counted
                    })
                    --
                    @variant("Badge: raise the count", bsn! {
                        ~{PushButton::from_label("Add one")}
                        on(add_one)
                    })
                    --
                    @variant("Badge: none", bsn! {
                        IconButton { icon: {plain}, name: "Stops", size: IconButtonSize::Large }
                    })
                    --
                    @variant("Badge: warning count", bsn! {
                        IconButton { icon: {warning}, name: "Warnings", size: IconButtonSize::Large }
                        ~{Badge::count(12).with_tone(BadgeTone::Warning)}
                    })
                ]
            ]
        },
    );
}

/// Marks the button whose badge "Add one" raises.
#[derive(Component, Default, Clone)]
pub(super) struct Counted;

/// Raises the count of the badge on the `Counted` button: a new count is a new `Badge`, and the
/// pill follows it.
fn add_one(
    _: On<Activate>,
    mut commands: Commands,
    badges: Query<(Entity, &Badge), With<Counted>>,
) {
    for (entity, badge) in &badges {
        let count = badge.text.parse::<usize>().unwrap_or(0);
        commands.entity(entity).insert(Badge {
            text: (count + 1).to_string(),
            ..badge.clone()
        });
    }
}
