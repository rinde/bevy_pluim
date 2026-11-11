//! The buttons page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn buttons(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Buttons")}
                --
                ~{groups()}
                Children [
                    @variant("Regular", bsn! { ~{PushButton::from_label("Regular")} })
                    --
                    @variant("Compact", bsn! {
                        ~{PushButton::from_label("Compact")}
                        ~{ButtonSize::Compact}
                    })
                    --
                    @variant("Disabled", bsn! {
                        ~{PushButton::from_label("Disabled")}
                        InteractionDisabled
                    })
                    --
                    @variant("Toggle (off, then on)", bsn! {
                        ~{row()}
                        Children [
                            ~{PushButton::from_label("Off")}
                            ~{ToggleState(false)}
                            --
                            ~{PushButton::from_label("On")}
                            ~{ToggleState(true)}
                        ]
                    })
                    --
                    @variant("Icon placement: left, right, top, bottom", bsn! {
                        ~{row()}
                        Children [
                            ~{PushButton::new("Left", "+")}
                            ~{IconPlacement::Left}
                            --
                            ~{PushButton::new("Right", "+")}
                            ~{IconPlacement::Right}
                            --
                            ~{PushButton::new("Top", "+")}
                            ~{IconPlacement::Top}
                            --
                            ~{PushButton::new("Bottom", "+")}
                            ~{IconPlacement::Bottom}
                        ]
                    })
                    --
                    @variant("Link", bsn! { ~{Link::new("Timetable")} })
                    --
                    @variant("Disabled link", bsn! {
                        ~{Link::new("Depot map")}
                        InteractionDisabled
                    })
                ]
            ]
        },
    );
}

pub(super) fn icon_buttons(
    spawner: In<StackChildSpawner>,
    icon: Res<Icon>,
    mut commands: Commands,
) {
    let icon = icon.0.clone();
    let (small, regular, large, toggle) = (icon.clone(), icon.clone(), icon.clone(), icon.clone());
    let (warning, danger, badge, disabled) = (icon.clone(), icon.clone(), icon.clone(), icon);
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Icon buttons")}
                --
                ~{groups()}
                Children [
                    @variant("Small", bsn! {
                        IconButton { icon: {small}, name: "Small", size: IconButtonSize::Small }
                    })
                    --
                    @variant("Regular", bsn! {
                        IconButton { icon: {regular}, name: "Regular", size: IconButtonSize::Regular }
                    })
                    --
                    @variant("Large", bsn! {
                        IconButton { icon: {large}, name: "Large", size: IconButtonSize::Large }
                    })
                    --
                    @variant("Toggle", bsn! {
                        IconButton { icon: {toggle}, name: "Toggle", size: IconButtonSize::Regular }
                        ~{ToggleState(false)}
                    })
                    --
                    @variant("Warning tone", bsn! {
                        IconButton { icon: {warning}, name: "Warning", size: IconButtonSize::Large, tone: Tone::Warning }
                    })
                    --
                    @variant("Danger tone", bsn! {
                        IconButton { icon: {danger}, name: "Danger", size: IconButtonSize::Regular, tone: Tone::Danger }
                    })
                    --
                    @variant("With a badge", bsn! {
                        IconButton { icon: {badge}, name: "Trains", size: IconButtonSize::Large }
                        ~{Badge::count(3)}
                    })
                    --
                    @variant("Disabled", bsn! {
                        IconButton { icon: {disabled}, name: "Disabled", size: IconButtonSize::Regular }
                        InteractionDisabled
                    })
                ]
                --
                ~{Label::caption("Every icon button shows its name as a tooltip after half a second.")}
            ]
        },
    );
}
