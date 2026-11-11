//! The combos page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

pub(super) fn combos(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Combos")}
                --
                ~{groups()}
                Children [
                    @variant("None or one, with a placeholder", bsn! {
                        ~{Combo}
                        ~{ComboPlaceholder("Pick a country".to_string())}
                        @{Selection::builder().zero_or_one().scene_with_items(countries())}
                    })
                    --
                    @variant("Exactly one", bsn! {
                        ~{Combo}
                        @{Selection::builder().one().scene_with_items(countries())}
                    })
                    --
                    @variant("A long list: 100 options, the popup scrolls", bsn! {
                        ~{Combo}
                        @{Selection::builder().zero_or_one().scene_with_items((1..=100).map(|n| Item::new(format!("Option {n}"))))}
                    })
                    --
                    @variant("Options with a note", bsn! {
                        ~{Combo}
                        @{Selection::builder().zero_or_one().scene_with_items(stations())}
                    })
                    --
                    @variant("One or more", bsn! {
                        ~{Combo}
                        @{Selection::builder().one_or_more().scene_with_items(countries())}
                    })
                    --
                    @variant("None or more", bsn! {
                        ~{Combo}
                        @{Selection::builder().zero_or_more().scene_with_items(countries())}
                    })
                ]
            ]
        },
    );
}

pub(super) fn filter_combos(spawner: In<StackChildSpawner>, mut commands: Commands) {
    let shared = commands
        .spawn(
            Selection::builder()
                .zero_or_one()
                .build_with_items(countries()),
        )
        .id();
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Filter combos")}
                --
                ~{Label::caption("Type to filter the options, Arrow Down and Up to move through the matches, Enter to pick.")}
                --
                ~{groups()}
                Children [
                    @variant("Shares its selection with the combo beside it", bsn! {
                        ~{row()}
                        Children [
                            ~{FilterCombo::default().with_name("Country")}
                            ~{Selection::reference(shared)}
                            --
                            ~{Combo}
                            ~{Selection::reference(shared)}
                        ]
                    })
                    --
                    @variant("Exactly one", bsn! {
                        ~{FilterCombo::default().with_name("Country")}
                        @{Selection::builder().one().scene_with_items(countries())}
                    })
                    --
                    @variant("None or more, stays open for the next pick", bsn! {
                        ~{FilterCombo::default().with_name("Stations")}
                        @{Selection::builder().zero_or_more().scene_with_items(countries())}
                    })
                    --
                    @variant("A long list: 80 options, the popup scrolls", bsn! {
                        ~{FilterCombo::default().with_name("Number")}
                        @{Selection::builder().zero_or_one().scene_with_items((1..=80).map(|n| Item::new(format!("Option {n}"))))}
                    })
                ]
            ]
        },
    );
}
