//! The navigation page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn navigation(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Tabs and trails")}
                --
                ~{groups()}
                Children [
                    @variant("Tab bar", bsn! {
                        TabBar
                        @{Selection::builder().one().scene_with_items(["Overview", "Trains", "Stops"].map(Item::new))}
                    })
                    --
                    @variant("Bread crumbs", bsn! {
                        ~{BreadCrumbs}
                        @{Selection::builder().one_or_more().scene_with_items(["Home", "Lines", "Line 4"].map(Item::new))}
                    })
                ]
            ]
        },
    );
}
