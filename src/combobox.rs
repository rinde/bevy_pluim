use bevy::ecs::spawn::SpawnIter;
use bevy::prelude::*;
use derive_more::Add;
use derive_more::AddAssign;

#[expect(clippy::wildcard_imports, reason = "ease of use of colors")]
use crate::colors::*;

/// Plugin that adds all combobox systems to the app.
#[derive(Debug, Copy, Clone)]
pub struct ComboBoxPlugin;

impl Plugin for ComboBoxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Widgets>().add_systems(
            Update,
            (
                toggle_combo_visibility,
                select_combo_item_system,
                close_combobox_on_outside_click_system,
                selectable_items_hover,
                hover_combobox_button_system,
            ),
        );
    }
}

#[derive(Resource, Default, Debug, Copy, Clone)]
pub struct Widgets {
    combos: ComboId,
}

#[derive(
    Deref, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Add, AddAssign, Component,
)]
pub struct ComboId(u8);

#[derive(Component, Debug, Clone)]
pub struct Combo {
    pub id: ComboId,
    pub options: Vec<String>,
    pub selected_index: Option<usize>,
    pub is_open: bool,
}

impl Combo {
    #[must_use]
    const fn new(id: ComboId, options: Vec<String>) -> Self {
        Self {
            id,
            options,
            selected_index: None,
            is_open: false,
        }
    }

    #[must_use]
    pub fn selected_text(&self) -> Option<&str> {
        self.selected_index
            .and_then(|index| self.options.get(index))
            .map(String::as_str)
    }

    pub const fn select_index(&mut self, index: usize) {
        if index < self.options.len() {
            self.selected_index = Some(index);
        }
    }

    pub const fn clear_selection(&mut self) {
        self.selected_index = None;
    }
}

/// Marker component for the main combobox button.
#[derive(Component, Debug, Copy, Clone)]
pub(crate) struct ComboButton;

/// Marker component for the dropdown container.
#[derive(Component, Debug, Copy, Clone)]
pub(crate) struct SelectableItems;

#[derive(Component, Debug, Copy, Clone)]
pub(crate) struct SelectableItem {
    index: usize,
    combo_id: ComboId,
}

/// Marker component for the text display area.
#[derive(Component, Debug, Copy, Clone)]
pub(crate) struct ComboDisplay;

/// Configuration for spawning a combobox.
#[derive(Debug, Clone)]
pub struct ComboBoxConfig {
    /// Label text to display above the combobox
    pub label: Option<String>,
    pub placeholder: String,
    pub options: Vec<String>,
    pub width: Val,
    pub initial_selection: Option<usize>,
}

impl Default for ComboBoxConfig {
    fn default() -> Self {
        Self {
            label: None,
            placeholder: "Select...".to_string(),
            options: Vec::new(),
            width: Val::Px(200.0),
            initial_selection: None,
        }
    }
}

impl Widgets {
    pub fn combobox(&mut self, config: ComboBoxConfig) -> impl Bundle {
        let combo_id = self.combos;
        self.combos += ComboId(1);
        let initial_text = if let Some(index) = config.initial_selection {
            config
                .options
                .get(index)
                .cloned()
                .unwrap_or_else(|| config.placeholder.clone())
        } else {
            config.placeholder.clone()
        };
        (
            Name::new(format!("ComboBox{}", *combo_id)),
            Node {
                width: config.width,
                flex_direction: FlexDirection::Column,
                position_type: PositionType::Relative,
                ..default()
            },
            Combo::new(combo_id, config.options.clone()),
            children![
                combo_button(combo_id, initial_text),
                selectable_items(combo_id, config.options.clone())
            ],
        )
    }
}

fn combo_button(combo_id: ComboId, initial_text: String) -> impl Bundle {
    (
        Button,
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(42.0),
            justify_content: JustifyContent::SpaceBetween,
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(BG_PRIMARY),
        BorderColor(BORDER_PRIMARY),
        BorderRadius::all(Val::Px(6.0)),
        ComboButton,
        combo_id,
        children![
            (
                Name::new("ComboBoxDisplay"),
                Text(initial_text),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(GRAY_900),
                ComboDisplay,
                combo_id
            ),
            (
                Text("\\/".to_string()),
                TextFont {
                    font_size: 12.0,
                    ..default()
                },
                TextColor(TEXT_MUTED),
            )
        ],
    )
}

fn selectable_items(combo_id: ComboId, options: Vec<String>) -> impl Bundle {
    const B_RADIUS: Val = Val::Px(10.0);
    let last = options.len() - 1;
    let children = options.into_iter().enumerate().map(move |(idx, option)| {
        (
            Button,
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(36.0),
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                ..default()
            },
            BorderRadius {
                top_left: if idx == 0 { B_RADIUS } else { Val::ZERO },
                top_right: if idx == 0 { B_RADIUS } else { Val::ZERO },
                bottom_left: if idx == last { B_RADIUS } else { Val::ZERO },
                bottom_right: if idx == last { B_RADIUS } else { Val::ZERO },
            },
            SelectableItem {
                index: idx,
                combo_id,
            },
            children![(
                Text(option),
                TextFont {
                    font_size: 14.0,
                    ..default()
                },
                TextColor(GRAY_900),
            )],
        )
    });
    let children = Children::spawn(SpawnIter(children));
    (
        Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            position_type: PositionType::Absolute,
            top: Val::Px(44.0),
            left: Val::Px(0.0),
            border: UiRect::all(Val::Px(1.0)),
            display: Display::None,
            ..default()
        },
        BackgroundColor(BG_PRIMARY),
        BorderColor(BORDER_SECONDARY),
        BorderRadius::all(B_RADIUS),
        GlobalZIndex(1000),
        BoxShadow::new(
            SHADOW_MD,
            Val::Px(2.0),
            Val::Px(4.0),
            Val::Px(0.0),
            Val::Px(8.0),
        ),
        SelectableItems,
        combo_id,
        children,
    )
}

/// System that handles clicking the combobox button to toggle the dropdown.
#[expect(clippy::type_complexity, reason = "bevy")]
pub(crate) fn toggle_combo_visibility(
    interaction_query: Query<(&Interaction, &ComboId), (Changed<Interaction>, With<ComboButton>)>,
    mut combobox_query: Query<&mut Combo>,
    mut select_items_query: Query<(&mut Node, &ComboId), With<SelectableItems>>,
    mut items_style_query: Query<(&mut BackgroundColor, &SelectableItem), With<SelectableItem>>,
) {
    for (interaction, interaction_id) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Find the combobox that contains this button
            for mut combobox in &mut combobox_query {
                if combobox.id == *interaction_id {
                    combobox.is_open = !combobox.is_open;

                    if combobox.is_open {
                        for (mut background_color, item) in &mut items_style_query {
                            if item.combo_id == combobox.id {
                                if combobox.selected_index.is_some_and(|idx| idx == item.index) {
                                    *background_color = BackgroundColor(BLUE_500);
                                } else {
                                    *background_color = BackgroundColor(BG_PRIMARY);
                                }
                            }
                        }
                    }

                    for (mut dropdown_style, select_id) in &mut select_items_query {
                        if *select_id == combobox.id {
                            dropdown_style.display = if combobox.is_open {
                                Display::Flex
                            } else {
                                Display::None
                            };
                            break;
                        }
                    }

                    break;
                }
            }
            break;
        }
    }
}

/// System that handles selecting items from the dropdown.
#[expect(clippy::type_complexity, reason = "bevy")]
pub(crate) fn select_combo_item_system(
    interaction_query: Query<
        (&Interaction, &SelectableItem),
        (Changed<Interaction>, With<SelectableItem>),
    >,
    mut combo_query: Query<&mut Combo>,
    mut dropdown_query: Query<&mut Node, With<SelectableItems>>,
    mut display_query: Query<(&mut Text, &ComboId), With<ComboDisplay>>,
) {
    for (interaction, item) in &interaction_query {
        if *interaction == Interaction::Pressed {
            // Update combobox selection
            for mut combo in &mut combo_query {
                if combo.id == item.combo_id {
                    combo.selected_index = Some(item.index);
                    combo.is_open = false;

                    for (mut text, id) in &mut display_query {
                        if combo.id == *id {
                            if let Some(selected_text) = combo.selected_text() {
                                (**text) = selected_text.to_string();
                            }
                            break;
                        }
                    }
                    break;
                }
            }

            // Hide dropdown
            for mut dropdown_style in &mut dropdown_query {
                dropdown_style.display = Display::None;
            }
        }
    }
}

pub(crate) fn selectable_items_hover(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &mut SelectableItem),
        Changed<Interaction>,
    >,
    combobox_query: Query<&Combo>,
) {
    for (interaction, mut background_color, item) in &mut interaction_query {
        let selected_index = combobox_query
            .iter()
            .find_map(|c| c.is_open.then_some(c.selected_index))
            .flatten();

        match (*interaction, selected_index) {
            (Interaction::Hovered, Some(x)) if x == item.index => {
                *background_color = BackgroundColor(BLUE_600);
            }
            (Interaction::Hovered, _) => {
                *background_color = BackgroundColor(BLUE_200);
            }
            (Interaction::None, Some(x)) if x == item.index => {
                *background_color = BackgroundColor(BLUE_500);
            }
            (Interaction::None, _) => {
                *background_color = BackgroundColor(BG_PRIMARY);
            }
            (Interaction::Pressed, _) => {
                *background_color = BackgroundColor(BLUE_500);
            }
        }
    }
}

/// System that closes the dropdown when clicking outside of it.
pub(crate) fn close_combobox_on_outside_click_system(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut combobox_query: Query<&mut Combo>,
    mut dropdown_query: Query<&mut Node, With<SelectableItems>>,
    combo_button_query: Query<&Interaction, With<ComboButton>>,
    item_query: Query<&Interaction, With<SelectableItem>>,
    mut combo_style_query: Query<
        (&mut BackgroundColor, &mut BorderColor, &ComboId),
        With<ComboButton>,
    >,
) {
    if !mouse_button.just_pressed(MouseButton::Left) {
        return;
    }
    // Check if click was on combobox button or item
    let clicked_on_combobox = combo_button_query
        .iter()
        .any(|&interaction| interaction == Interaction::Pressed)
        || item_query
            .iter()
            .any(|&interaction| interaction == Interaction::Pressed);

    if !clicked_on_combobox {
        for mut combobox in &mut combobox_query {
            if combobox.is_open {
                combobox.is_open = false;
            }
        }

        for mut dropdown_style in &mut dropdown_query {
            dropdown_style.display = Display::None;
        }
    }
    for (mut background_color, mut border_color, _) in &mut combo_style_query {
        *background_color = BackgroundColor(BG_PRIMARY);
        *border_color = BorderColor(BORDER_PRIMARY);
    }
}

/// System that provides hover effects for the main combobox button.
#[expect(clippy::type_complexity, reason = "bevy")]
pub(crate) fn hover_combobox_button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &ComboId,
        ),
        (Changed<Interaction>, With<ComboButton>),
    >,
    combobox_query: Query<&Combo>,
) {
    for (interaction, mut background_color, mut border_color, id) in &mut interaction_query {
        if let Some(combo) = combobox_query.iter().find(|c| c.id == *id) {
            match (*interaction, combo.is_open) {
                (Interaction::Hovered, _) | (_, true) => {
                    *background_color = BackgroundColor(GRAY_50);
                    *border_color = BorderColor(BLUE_400);
                }
                (Interaction::None, false) => {
                    *background_color = BackgroundColor(BG_PRIMARY);
                    *border_color = BorderColor(BORDER_PRIMARY);
                }
                (Interaction::Pressed, false) => {}
            }
        }
    }
}
