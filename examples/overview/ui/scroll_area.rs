//! The scroll area page of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;
use bevy::color::palettes::tailwind::{
    AMBER_600, LIME_600, PURPLE_600, ROSE_600, SKY_600, YELLOW_600,
};

pub(super) fn scroll_areas(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Scroll area")}
                --
                ~{groups()}
                Children [
                    @variant("Vertical", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @list_scroll_area(true, false)
                    })
                    --
                    @variant("Vertical and horizontal", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @list_scroll_area(true, true)
                    })
                    --
                    @variant("Squares, vertical and horizontal", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @squares_area(300, GridAutoFlow::Row, true, true)
                    })
                    --
                    @variant("Squares, vertical only", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @squares_area(50, GridAutoFlow::Row, true, false)
                    })
                    --
                    @variant("Squares in columns, both axes", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @squares_area(200, GridAutoFlow::Column, true, true)
                    })
                    --
                    @variant("Squares, horizontal only", bsn! {
                        Node { width: px(200.0), height: px(200.0) }
                        @squares_area(50, GridAutoFlow::Column, false, true)
                    })
                ]
            ]
        },
    );
}

/// A scroll area over a grid of `num` coloured squares.
fn squares_area(num: usize, grid_auto_flow: GridAutoFlow, v: bool, h: bool) -> impl Scene {
    ScrollArea::scene(move || bsn_list! { @squares(num, grid_auto_flow) }, v, h)
}

/// A grid of `num` coloured squares, laid out along `grid_auto_flow`.
fn squares(num: usize, grid_auto_flow: GridAutoFlow) -> impl Scene {
    const COLORS: [Srgba; 6] = [
        YELLOW_600, SKY_600, ROSE_600, PURPLE_600, LIME_600, AMBER_600,
    ];
    let squares = (0..num)
        .map(|i| {
            let color = COLORS
                .get(i % COLORS.len())
                .map_or(Color::NONE, |c| Color::Srgba(*c));
            bsn! {
                Node {
                    width: px(50.0),
                    height: px(50.0),
                }
                Pickable {
                    should_block_lower: false,
                }
                BackgroundColor(color)
            }
        })
        .collect::<Vec<_>>();
    bsn! {
        Pickable {
            should_block_lower: false,
        }
        Node {
            display: Display::Grid,
            grid_template_columns: vec![RepeatedGridTrack::auto(7)],
            grid_template_rows: vec![RepeatedGridTrack::auto(7)],
            grid_auto_flow,
            justify_content: JustifyContent::SpaceBetween,
            align_content: AlignContent::FlexStart,
            row_gap: px(2.0),
            column_gap: px(2.0),
        }
        Children [ {squares} ]
    }
}

/// A scroll area over a list of many countries.
pub(super) fn list_scroll_area(vertical: bool, horizontal: bool) -> impl Scene {
    ScrollArea::scene(
        || {
            let items = (1..=20).map(|i| Item::new(format!("A rather long country name {i}")));
            bsn_list! {
                List
                @{Selection::builder().zero_or_one().scene_with_items(items)}
            }
        },
        vertical,
        horizontal,
    )
}
