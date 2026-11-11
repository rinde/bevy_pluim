//! The toolbars page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn toolbars(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let tools = |icon: &Handle<Image>| {
        vec![
            Item {
                data: Tool::in_group(icon.clone(), "Select", 0).with_key_hint("1"),
            },
            Item {
                data: Tool::in_group(icon.clone(), "Rail", 1).with_key_hint("2"),
            },
            Item {
                data: Tool::in_group(icon.clone(), "Signal", 1).with_key_hint("3"),
            },
            Item {
                data: Tool::in_group(icon.clone(), "Bulldoze", 2).with_tone(Tone::Danger),
            },
        ]
    };
    let toned = vec![
        Item {
            data: Tool::in_group(icon.0.clone(), "Neutral", 0),
        },
        Item {
            data: Tool::in_group(icon.0.clone(), "Warning", 1).with_tone(Tone::Warning),
        },
        Item {
            data: Tool::in_group(icon.0.clone(), "Danger", 2).with_tone(Tone::Danger),
        },
    ];
    let (filled, hidden, underlined) = (tools(&icon.0), tools(&icon.0), tools(&icon.0));
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Toolbar")}
                --
                ~{groups()}
                Children [
                    @variant("Filled, with key hints and a danger tool", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Filled)}
                        @{Selection::builder().one().scene_with_items(filled)}
                    })
                    --
                    @variant("Filled, labels hidden", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Filled)}
                        ~{ToolbarLabels::Hidden}
                        @{Selection::builder().one().scene_with_items(hidden)}
                    })
                    --
                    @variant("Underlined, none or one, the label is the tooltip", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Underlined)}
                        @{Selection::builder().zero_or_one().scene_with_items(underlined)}
                    })
                    --
                    @variant("Tool tones: neutral, warning, danger", bsn! {
                        ~{Toolbar::new(ToolbarStyle::Filled)}
                        @{Selection::builder().one().scene_with_items(toned)}
                    })
                ]
            ]
        },
    );
}

pub(super) fn galleries(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let cards = [
        ("Steam", "Locomotives"),
        ("Diesel", "Locomotives"),
        ("Electric", "Locomotives"),
        ("Passenger", "Wagons"),
        ("Box wagon", "Wagons"),
        ("Tank wagon", "Wagons"),
    ]
    .map(|(label, group)| Item {
        data: GalleryCard::new(icon.0.clone(), label, group),
    });
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Gallery")}
                --
                ~{groups()}
                Children [
                    @variant("Two groups of cards, in a horizontal scroll area", bsn! {
                        Node { width: px(600.0) }
                        @gallery_scroll_area(cards)
                    })
                ]
            ]
        },
    );
}

/// A horizontal scroll area over the gallery of `cards`.
pub(super) fn gallery_scroll_area(cards: [Item<GalleryCard>; 6]) -> impl Scene {
    ScrollArea::scene(
        move || {
            bsn_list! {
                Gallery
                @{Selection::builder().one().scene_with_items(cards.clone())}
            }
        },
        false,
        true,
    )
}
