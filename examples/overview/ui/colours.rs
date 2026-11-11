//! The colours page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn colours(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let swatches = || {
        COLORS
            .iter()
            .filter(|c| c.variant == bevy_pluim::tailwind::Variant::V500)
            .map(|c| Item { data: c.color })
            .collect::<Vec<_>>()
    };
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Colours")}
                --
                ~{groups()}
                Children [
                    @variant("Palette, exactly one", bsn! {
                        ~{ColorPalette}
                        ~{ListLayout::VerticalGrid { columns: 11 }}
                        @{Selection::builder().one().scene_with_items(swatches())}
                    })
                    --
                    @variant("Palette in a scroll area", bsn! {
                        Node { width: px(250.0), height: px(120.0) }
                        @palette_scroll_area(swatches())
                    })
                    --
                    @variant("Palette combo, none or one", bsn! {
                        ColorPaletteCombo
                        ~{ListLayout::VerticalGrid { columns: 11 }}
                        @{Selection::builder().zero_or_one().scene_with_items(swatches())}
                    })
                    --
                    @variant("Palette combo, exactly one", bsn! {
                        ColorPaletteCombo
                        ~{ListLayout::VerticalGrid { columns: 11 }}
                        @{Selection::builder().one().scene_with_items(swatches())}
                    })
                    --
                    @variant("Palette combo in a scroll area", bsn! {
                        Node { width: px(100.0), height: px(100.0) }
                        @combo_scroll_area(swatches())
                    })
                    --
                    @variant("Palette combos added and removed", bsn! {
                        Node { flex_direction: FlexDirection::Column, row_gap: px(8.0) }
                        Children [
                            Node { column_gap: px(8.0) }
                            Children [
                                ~{PushButton::from_label("+")}
                                on(add_combo)
                                --
                                ~{PushButton::from_label("-")}
                                on(remove_combo)
                            ]
                            --
                            Node
                            ExtraComponentsArea
                        ]
                    })
                ]
            ]
        },
    );
}

/// Holds the palette combos the "+" button adds.
#[derive(Component, Default, Clone)]
pub(super) struct ExtraComponentsArea;

fn add_combo(
    _: On<Activate>,
    area: Single<Entity, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    let swatches = COLORS
        .iter()
        .filter(|c| c.variant == bevy_pluim::tailwind::Variant::V500)
        .map(|c| Item { data: c.color });
    commands.entity(*area).with_children(|parent| {
        parent.spawn((
            ColorPaletteCombo,
            ListLayout::VerticalGrid { columns: 11 },
            Selection::builder()
                .zero_or_one()
                .build_with_items(swatches),
        ));
    });
}

fn remove_combo(
    _: On<Activate>,
    area: Single<&Children, With<ExtraComponentsArea>>,
    mut commands: Commands,
) {
    if let Some(last) = area.iter().last() {
        commands.entity(last).despawn();
    }
}

/// A scroll area over a palette combo and a card, to see the popup is not clipped.
fn combo_scroll_area(swatches: Vec<Item<Color>>) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                ColorPaletteCombo
                ~{ListLayout::VerticalGrid { columns: 11 }}
                @{Selection::builder().zero_or_one().scene_with_items(swatches.clone())}
                --
                Node { width: px(100.0), height: px(100.0) }
                ~{Card}
            }
        },
        true,
        true,
    )
}

/// A scroll area over a palette of `swatches`.
pub(super) fn palette_scroll_area(swatches: Vec<Item<Color>>) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                ~{ColorPalette}
                ~{ListLayout::VerticalGrid { columns: 6 }}
                @{Selection::builder().one().scene_with_items(swatches.clone())}
            }
        },
        true,
        false,
    )
}
