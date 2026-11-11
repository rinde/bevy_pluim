//! The feedback page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn feedback(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Tooltip and toast")}
                --
                ~{groups()}
                Children [
                    @variant("Tooltip on a push button", bsn! {
                        ~{PushButton::from_label("Hover me")}
                        ~{Tooltip::new("A tooltip appears after half a second")}
                    })
                    --
                    @variant("The tooltip an icon button carries", bsn! {
                        IconButton { icon: {icon}, name: "Hover me too", size: IconButtonSize::Regular }
                    })
                    --
                    @variant("Toast", bsn! {
                        ~{PushButton::from_label("Raise a toast")}
                        on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
                            commands.spawn((Toast::new("Saved to rails.ron"), ChildOf(*root)));
                        })
                    })
                    --
                    @variant("Toast: deleted", bsn! {
                        ~{PushButton::from_label("Delete")}
                        on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
                            commands.spawn((Toast::new("Deleted 3 pieces"), ChildOf(*root)));
                        })
                    })
                    --
                    @variant("Toast: refused", bsn! {
                        ~{PushButton::from_label("Refuse")}
                        on(|_: On<Activate>, mut commands: Commands, root: Single<Entity, With<UiRoot>>| {
                            commands.spawn((Toast::new("The platform cannot grow that way"), ChildOf(*root)));
                        })
                    })
                ]
            ]
        },
    );
}
