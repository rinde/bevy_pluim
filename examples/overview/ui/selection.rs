//! The selection page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn segmented(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let icon = icon.0.clone();
    let with_icon = move |text| (Item::new(text), ItemIcon(icon.clone()));
    let (text_and_icon, icon_only) = (
        ["Grid", "List"].map(with_icon.clone()),
        ["Map", "Table"].map(with_icon),
    );
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Segmented control")}
                --
                ~{groups()}
                Children [
                    @variant("Two segments", bsn! {
                        SegmentedControl
                        @{Selection::builder().one().scene_with_items(["Auto", "Manual"].map(Item::new))}
                    })
                    --
                    @variant("Three segments", bsn! {
                        SegmentedControl
                        @{Selection::builder().one().scene_with_items(["Dark", "System", "Light"].map(Item::new))}
                    })
                    --
                    @variant("Text and icon", bsn! {
                        SegmentedControl
                        @{Selection::builder().one().scene_with_items(text_and_icon)}
                    })
                    --
                    @variant("Icon only, the text is the tooltip", bsn! {
                        SegmentedControl
                        ~{SegmentLabels::Hidden}
                        @{Selection::builder().one().scene_with_items(icon_only)}
                    })
                ]
            ]
        },
    );
}

pub(super) fn light_dark(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Light dark toggle")}
                --
                ~{groups()}
                Children [
                    @variant("Light, system or dark; the system follows the operating system", bsn! {
                        LightDarkToggle
                    })
                ]
                --
                ~{Label::caption("The toggle above the tabs is this widget, and it switches the theme of this overview.")}
            ]
        },
    );
}

pub(super) fn chips(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let items = || {
        vec![
            Item {
                data: Chip::with_count("All", 24),
            },
            Item {
                data: Chip::with_count("Running", 12),
            },
            Item {
                data: Chip::with_count("Waiting", 7),
            },
            Item {
                data: Chip::with_count("Problems", 3),
            },
            Item {
                data: Chip::new("Depot"),
            },
        ]
    };
    let shared = commands
        .spawn(Selection::builder().one().build_with_items(items()))
        .id();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Filter chips")}
                --
                ~{groups()}
                Children [
                    @variant("With counts", bsn! {
                        FilterChips
                        ~{Selection::reference(shared)}
                    })
                    --
                    @variant("Narrow, so that the chips wrap", bsn! {
                        FilterChips
                        Node {
                            width: px(220.0),
                            flex_wrap: FlexWrap::Wrap,
                            column_gap: px(6.0),
                            row_gap: px(6.0),
                        }
                        @{Selection::builder().one().scene_with_items(items())}
                    })
                    --
                    @variant("A second control sharing the selection of the first", bsn! {
                        FilterChips
                        ~{Selection::reference(shared)}
                    })
                ]
            ]
        },
    );
}
