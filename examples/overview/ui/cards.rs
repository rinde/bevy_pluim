//! The cards page(s) of the overview.

#[cfg_attr(
    not(test),
    expect(
        clippy::wildcard_imports,
        reason = "the pages share the imports of `ui`; clippy exempts it in a test target"
    )
)]
use super::*;

/// How many lines of text the body of a panel gets.
#[derive(Component, Default, Clone)]
pub(super) struct Lines(usize);

/// Marks a panel or setting row whose contents have been spawned.
#[derive(Component)]
pub(super) struct Filled;

#[derive(Component, Default, Clone)]
pub(super) struct ControlLines;

pub(super) fn cards(spawner: In<StackChildSpawner>, icon: Res<Icon>, mut commands: Commands) {
    let (close, close_scrolling) = (icon.0.clone(), icon.0.clone());
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Cards and panels")}
                --
                ~{groups()}
                Children [
                    @variant("Card", bsn! {
                        ~{Card}
                        Node {
                            padding: UiRect::all(px(12.0)),
                            row_gap: px(8.0),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::FlexStart,
                        }
                        Children [
                            ~{Label::title("Card")}
                            --
                            ~{Label::secondary("A themed surface")}
                            --
                            ~{PushButton::from_label("Button on a card")}
                        ]
                    })
                    --
                    @variant("Panel: closable, with a subtitle", bsn! {
                        Panel { title: "Trains", subtitle: {Some("3 open".to_string())}, close_icon: {HandleTemplate::Handle(close)} }
                        Lines(3)
                    })
                    --
                    // A body past 150 pixels scrolls.
                    @variant("Panel: the body scrolls past 150 px", bsn! {
                        Panel { title: "Stops", close_icon: {HandleTemplate::Handle(close_scrolling.clone())}, max_body_height: 150.0 }
                        Lines(12)
                    })
                    --
                    @variant("Panel: scrolling controls", bsn! {
                        Panel { title: "Controls", close_icon: {HandleTemplate::Handle(close_scrolling)}, max_body_height: 180.0 }
                        ControlLines
                    })
                ]
            ]
        },
    );
}

/// Fills the bodies of the panels once they are spawned.
pub(super) fn fill_panels(
    mut commands: Commands,
    panels: Query<(Entity, &PanelRefs, &Lines), Without<Filled>>,
) {
    for (panel, refs, lines) in &panels {
        commands.entity(panel).insert(Filled);
        for line in 1..=lines.0 {
            commands.spawn((Label::regular(format!("Line {line}")), ChildOf(refs.body())));
        }
    }
}

#[expect(
    clippy::expect_used,
    reason = "the panel uses a fixed valid value range"
)]
pub(super) fn fill_control_panels(
    mut commands: Commands,
    panels: Query<(Entity, &PanelRefs), (With<ControlLines>, Without<Filled>)>,
) {
    for (panel, refs) in &panels {
        commands.entity(panel).insert(Filled);
        for index in 0..12 {
            let step_value = Value::new_u8(index, 0, 20, 1).expect("a valid 0 to 20 range");
            let group = commands
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: px(6.0),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ChildOf(refs.body()),
                ))
                .id();
            let row = commands
                .spawn((
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: px(8.0),
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    ChildOf(group),
                ))
                .id();
            commands.spawn((
                PushButton::from_label(format!("Action {}", index + 1)),
                ChildOf(row),
            ));
            commands.spawn((
                Combo,
                Selection::builder()
                    .zero_or_one()
                    .build_with_items(["Red", "White", "Blue", "Yellow"].map(Item::new)),
                ChildOf(row),
            ));
            if index == 0 {
                commands.spawn((
                    Combo,
                    ComboPlaceholder("Pick an option".to_string()),
                    Selection::builder()
                        .zero_or_one()
                        .build_with_items((1..=100).map(|n| Item::new(format!("Option {n}")))),
                    ChildOf(group),
                ));
            }
            commands.spawn((valid_slider(0.5, 0.0, 1.0, None), ChildOf(group)));
            commands.spawn((NumericStepper::default(), step_value, ChildOf(group)));
        }
    }
}

/// The control a setting row gets in its slot.
#[derive(Component, Default, Clone, Copy)]
pub(super) enum Control {
    #[default]
    Appearance,
    TextSize,
    Reset,
}

pub(super) fn setting_rows(spawner: In<StackChildSpawner>, mut commands: Commands) {
    spawner.spawn_scene(
        &mut commands,
        bsn! {
            #Page
            ~{page()}
            Children [
                ~{Label::header("Setting rows")}
                --
                ~{groups()}
                Children [
                    @variant("Rows with a segmented control, a stepper and a button", bsn! {
                        SettingRows
                        Node { width: px(420.0) }
                        Children [
                            ~{SettingRow::new("Appearance", "Light or dark")} Control::Appearance
                            --
                            ~{SettingRow::new("Text size", "In points")} Control::TextSize
                            --
                            ~{SettingRow::new("Reset", "Back to the defaults")} Control::Reset
                        ]
                    })
                ]
            ]
        },
    );
}

/// Spawns the control of each setting row into its slot once the row is spawned.
pub(super) fn fill_setting_rows(
    mut commands: Commands,
    rows: Query<(Entity, &SettingRowRefs, &Control), Without<Filled>>,
) -> Result {
    for (row, refs, control) in &rows {
        commands.entity(row).insert(Filled);
        let slot = refs.control_slot;
        match control {
            Control::Appearance => {
                commands.spawn((
                    SegmentedControl,
                    Selection::builder()
                        .one()
                        .build_with_items(["Light", "Dark"].map(Item::new)),
                    ChildOf(slot),
                ));
            }
            Control::TextSize => {
                commands.spawn((
                    NumericStepper::default(),
                    Value::new_u8(12, 8, 20, 1)?,
                    ChildOf(slot),
                ));
            }
            Control::Reset => {
                commands.spawn((PushButton::from_label("Reset"), ChildOf(slot)));
            }
        }
    }
    Ok(())
}
