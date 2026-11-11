//! The lists page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn lists(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Lists")}
                --
                ~{groups()}
                Children [
                    @variant("One selected", bsn! {
                        List
                        @{Selection::builder().one().scene_with_items(countries())}
                    })
                    --
                    @variant("Separated, many selected", bsn! {
                        List
                        ~{CornerStyle::Separated}
                        @{Selection::builder().zero_or_more().scene_with_items(countries())}
                    })
                    --
                    @variant("Two-column grid, tabs corners, none or one", bsn! {
                        List
                        ~{ListLayout::VerticalGrid { columns: 2 }}
                        ~{CornerStyle::Tabs}
                        @{Selection::builder().zero_or_one().scene_with_items(countries())}
                    })
                    --
                    // Ten thousand rows, of which only those in view are spawned.
                    @variant("Virtual list, 10,000 rows", bsn! {
                        Node { width: px(220.0), height: px(260.0) }
                        @virtual_list_scroll_area(VirtualList::new(row_height(28.0)))
                    })
                    --
                    @variant("Virtual list, separated", bsn! {
                        Node { width: px(220.0), height: px(260.0) }
                        @virtual_list_scroll_area(
                            VirtualList::separated(row_height(44.0)).unwrap_or_else(|_| std::process::abort())
                        )
                    })
                ]
            ]
        },
    );
}

/// A scroll area over a `VirtualList` of ten thousand rows.
pub(super) fn virtual_list_scroll_area(list: VirtualList) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                ~{list}
                @{Selection::builder().one().scene_with_items((1..=10_000).map(|row| Item::new(format!("Row {row}"))))}
            }
        },
        true,
        false,
    )
}

pub(super) fn row_height(height: f32) -> RowHeight {
    RowHeight::new(height).unwrap_or_else(|_| std::process::abort())
}
