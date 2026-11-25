//! `Combo`: a button that opens a [`List`] in a [`Float`].

use bevy::a11y::AccessibilityNode;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::input::keyboard::KeyboardInput;
use bevy::input_focus::{FocusLost, FocusedInput, InputFocus};
use bevy::picking::hover::Hovered;
use bevy::picking::pointer::PointerInput;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::{ComputedNode, UiGlobalTransform, UiSystems};
use bevy::ui_widgets::Activate;
use itertools::Itertools;

use crate::MouseEvent;
use crate::float::{Float, UiViewport, fit_vertically, keep_in_window_x};
use crate::focus::{AccessibilitySync, popup_focus_lost, popup_key_closes};
use crate::glyph_images::{GlyphKey, shared};
use crate::icon::Icon;
use crate::list::{List, NoDefaultItems};
use crate::popup::{self, ButtonLook, PopupPointers};
use crate::push_button::{NoDefaultHover, PushButton, PushButtonLabel, PushButtonRefs};
use crate::scroll_area::{popup_scroll_bar, step_flings, update_scroll_position};
use crate::select::{Item, Selection, SelectionParam, SelectionValue};
use crate::theme::Theme;

/// Registers the systems and observers of [`Combo`].
#[derive(Debug, Copy, Clone)]
pub struct ComboPlugin;

impl Plugin for ComboPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                close_combos_on_outside_click,
                selection_change,
                item_text_change,
            ),
        )
        .add_systems(
            Update,
            close_on_ancestor_scroll
                .after(update_scroll_position)
                .after(step_flings),
        )
        .add_systems(
            PostUpdate,
            (
                sync_accessibility.in_set(AccessibilitySync),
                place_open_lists.before(UiSystems::Layout),
            ),
        )
        .add_observer(on_insert_combo)
        .add_observer(on_insert_placeholder);
    }
}

/// A button that opens a [`List`] in a [`Float`].
///
/// The popup closes on an outside click, Escape, lost focus or an ancestor scrolling.
#[derive(Component, Debug, Clone, Copy, Reflect)]
#[reflect(Component)]
#[component(on_add=Self::on_add)]
#[require(
    Node {
        width: Val::Px(200.0),
        flex_direction: FlexDirection::Column,
        position_type: PositionType::Relative,
        ..default()
    },
    Name::new("Combo"),
)]
pub struct Combo;

/// Marks the button of a [`Combo`].
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Name::new("ComboButton"))]
pub struct ComboButton;

/// The text a [`Combo`]'s button shows while nothing is selected, "Select.." by default.
///
/// Removing it leaves the button as is; insert `ComboPlaceholder::default()` to restore the default.
#[derive(Component, Debug, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ComboPlaceholder(pub String);

impl Default for ComboPlaceholder {
    fn default() -> Self {
        Self(PLACEHOLDER_TEXT.to_string())
    }
}

/// The parts of a [`Combo`]. Only the widget builds it.
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct ComboRefs {
    button_id: Entity,
    list_id: Entity,
}

impl ComboRefs {
    /// The button.
    #[must_use]
    pub const fn button(&self) -> Entity {
        self.button_id
    }

    /// The popup list.
    #[must_use]
    pub const fn list(&self) -> Entity {
        self.list_id
    }
}

/// Marks the popup list of a [`Combo`].
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[require(Name::new("ComboList"), Hovered)]
pub struct ComboList;

/// Whether the popup of a [`Combo`] is showing.
#[derive(Component, Debug, Copy, Clone, Eq, PartialEq, Reflect)]
#[reflect(Component)]
#[component(immutable, on_insert=Self::on_insert)]
pub enum ComboState {
    /// The popup is showing.
    Open,
    /// The popup is hidden.
    Closed,
}

impl ComboState {
    /// The other state.
    #[must_use]
    pub const fn inverse(&self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }

    fn on_insert(mut world: DeferredWorld, context: HookContext) {
        let entity = world.entity(context.entity);
        let (Some(&state), Some(&ComboRefs { list_id, button_id })) =
            (entity.get::<Self>(), entity.get::<ComboRefs>())
        else {
            return;
        };

        let open = state == Self::Open;
        popup::show_popup(&mut world.commands(), open, list_id, list_id);
        if let Some(mut focus) = world.get_resource_mut::<InputFocus>() {
            popup::move_focus(&mut focus, open, list_id, button_id);
        }
        if let Some(mut node) = world.get_mut::<AccessibilityNode>(button_id) {
            popup::set_expanded(&mut node, open);
        }
    }
}

pub(crate) const PLACEHOLDER_TEXT: &str = "Select..";

pub(crate) const CHEVRON_SIZE: f32 = 16.0;

pub(crate) fn chevron() -> Image {
    const SIZE: u32 = 16;
    let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 5..11u32 {
        let half_width = 6 - (y - 5);
        for x in (8 - half_width)..(8 + half_width) {
            let at = ((y * SIZE + x) * 4) as usize;
            if let Some(pixel) = data.get_mut(at..at + 4) {
                pixel.copy_from_slice(&[255; 4]);
            }
        }
    }
    Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

impl Combo {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let mut button_id = Entity::PLACEHOLDER;
        let mut list_id = Entity::PLACEHOLDER;
        let placeholder = world
            .get::<ComboPlaceholder>(context.entity)
            .map_or_else(|| PLACEHOLDER_TEXT.to_string(), |p| p.0.clone());
        let chevron = shared(&mut world, GlyphKey::Chevron, chevron);
        world.commands().entity(context.entity).with_children(|x| {
            button_id = x
                .spawn((
                    PushButton::from_label(placeholder),
                    ComboButton,
                    NoDefaultHover,
                    Pickable {
                        should_block_lower: false,
                        ..default()
                    },
                ))
                .with_children(|button| {
                    button.spawn((
                        Icon::bundle(chevron, CHEVRON_SIZE),
                        Name::new("Chevron"),
                        Pickable::IGNORE,
                    ));
                })
                .id();

            x.spawn(Node::DEFAULT).with_children(|spawner| {
                list_id = spawner
                    .spawn((
                        List,
                        Float,
                        // `List` ignores the pointer; without this a click would reach what lies below.
                        Pickable::default(),
                        Visibility::Hidden,
                        ComboList,
                    ))
                    .id();
            });
        });

        world.commands().spawn(popup_scroll_bar(list_id));

        let combo_id = context.entity;
        world
            .commands()
            .entity(list_id)
            .observe(close_on_key(combo_id))
            .observe(close_on_focus_lost(combo_id));
        world
            .commands()
            .entity(combo_id)
            .insert((ComboState::Closed, ComboRefs { button_id, list_id }))
            .observe(handle_combo_button_mouse_events);
        world.commands().entity(button_id).observe(
            move |_: On<Activate>, states: Query<&ComboState>, mut commands: Commands| {
                if let Ok(state) = states.get(combo_id) {
                    commands.entity(combo_id).insert(state.inverse());
                }
            },
        );
    }
}

fn close_on_key(
    combo_id: Entity,
) -> impl Fn(On<FocusedInput<KeyboardInput>>, Query<&ComboState>, SelectionParam, Commands) {
    move |input, states, selections, mut commands| {
        if states.get(combo_id) == Ok(&ComboState::Open)
            && popup_key_closes(&input.event().input, selections.get(combo_id))
        {
            commands.entity(combo_id).insert(ComboState::Closed);
        }
    }
}

fn close_on_focus_lost(
    combo_id: Entity,
) -> impl Fn(On<FocusLost>, Query<(&ComboState, &ComboRefs)>, PopupPointers, ResMut<InputFocus>, Commands)
{
    move |lost, states, pointers, mut focus, mut commands| {
        let Ok((&state, sub_refs)) = states.get(combo_id) else {
            return;
        };
        // The event bubbles: only the list itself losing the focus counts.
        if lost.event().entity != sub_refs.list_id {
            return;
        }
        // The release on the button toggles the popup; closing here would reopen it.
        if state == ComboState::Open && pointers.pressed_over(sub_refs.button_id) {
            return;
        }
        let popup_hovered = pointers.over(None, sub_refs.list_id);
        if popup_focus_lost(
            &mut focus,
            state == ComboState::Open,
            sub_refs.button_id,
            sub_refs.list_id,
            popup_hovered,
        ) {
            commands.entity(combo_id).insert(ComboState::Closed);
        }
    }
}

fn close_on_ancestor_scroll(
    combos: Query<(Entity, &ComboState), With<Combo>>,
    parents: Query<&ChildOf>,
    scrolled: Query<(), Changed<ScrollPosition>>,
    mut commands: Commands,
) {
    for (entity, state) in &combos {
        if *state == ComboState::Open
            && parents
                .iter_ancestors(entity)
                .any(|ancestor| scrolled.contains(ancestor))
        {
            commands.entity(entity).insert(ComboState::Closed);
        }
    }
}

fn place_open_lists(
    viewport: UiViewport,
    combos: Query<(&ComboState, &ComboRefs)>,
    buttons: Query<(&ComputedNode, &UiGlobalTransform), With<ComboButton>>,
    mut lists: Query<(&mut Node, &ComputedNode), With<ComboList>>,
) {
    for (state, sub_refs) in &combos {
        if *state != ComboState::Open {
            continue;
        }
        let (Ok((button_computed, button_transform)), Ok((mut node, list_computed))) = (
            buttons.get(sub_refs.button_id),
            lists.get_mut(sub_refs.list_id),
        ) else {
            continue;
        };
        let Some(target) = viewport.size_of(sub_refs.button_id) else {
            continue;
        };
        let scale = button_computed.inverse_scale_factor();
        let button_size = button_computed.size() * scale;
        let button_center = button_transform.translation * scale;
        let list_size = list_computed.size() * list_computed.inverse_scale_factor();
        let left = Val::Px(keep_in_window_x(
            button_center.x - button_size.x / 2.0,
            list_size.x,
            target.x,
        ));
        let fit = fit_vertically(
            button_center.y - button_size.y / 2.0,
            button_center.y + button_size.y / 2.0,
            0.0,
            list_size.y,
            target.y,
        );
        let top = Val::Px(fit.top);
        let max_height = Val::Px(fit.max_height);
        if node.left != left
            || node.top != top
            || node.max_height != max_height
            || node.overflow.y != OverflowAxis::Scroll
        {
            node.left = left;
            node.top = top;
            node.max_height = max_height;
            node.overflow.y = OverflowAxis::Scroll;
        }
    }
}

fn on_insert_combo(t: On<Insert<Combo>>, selections: SelectionParam, mut commands: Commands) {
    let combo_ref = t.event().event_target();
    let Some(combo_selection) = selections.get_ref(combo_ref) else {
        return;
    };
    commands.run_system_cached_with(update_selection, (combo_selection, combo_ref));
}

fn on_insert_placeholder(
    t: On<Insert<ComboPlaceholder>>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    let combo_ref = t.event().event_target();
    let Some(combo_selection) = selections.get_ref(combo_ref) else {
        return;
    };
    commands.run_system_cached_with(update_selection, (combo_selection, combo_ref));
}

fn selection_change(
    q: Query<Entity, Changed<Selection>>,
    combos: Query<(Entity, &Selection), (With<Combo>, Without<NoDefaultItems>)>,
    mut commands: Commands,
) {
    for changed_selection_ref in q {
        for (combo_ref, combo_selection) in combos {
            let is_changed = match combo_selection {
                Selection::Ref(entity) => changed_selection_ref == *entity,
                Selection::Value(_selection) => changed_selection_ref == combo_ref,
            };
            if is_changed {
                commands
                    .run_system_cached_with(update_selection, (changed_selection_ref, combo_ref));
            }
        }
    }
}

fn item_text_change(
    renamed: Query<Entity, Changed<Item<String>>>,
    combos: Query<Entity, (With<Combo>, Without<NoDefaultItems>)>,
    selections: SelectionParam,
    mut commands: Commands,
) {
    if renamed.is_empty() {
        return;
    }
    let renamed: Vec<Entity> = renamed.iter().collect();
    for combo in &combos {
        if let Some((selection_ref, selection)) = selections.get_with_ref(combo)
            && selection.iter().any(|item| renamed.contains(&item))
        {
            commands.run_system_cached_with(update_selection, (selection_ref, combo));
        }
    }
}

fn update_selection(
    input: In<(Entity, Entity)>,
    selections: SelectionParam,
    combo_sub_refs: Query<&ComboRefs>,
    pblr: Query<&PushButtonRefs>,
    items: Query<&Item<String>>,
    placeholders: Query<&ComboPlaceholder>,
    mut labels: Query<&mut Text, With<PushButtonLabel>>,
    mut commands: Commands,
) {
    let (selection_ref, combo_ref) = *input;
    let (Ok(sub_refs), Some(selection)) =
        (combo_sub_refs.get(combo_ref), selections.get(selection_ref))
    else {
        return;
    };

    if selections
        .get_ref(sub_refs.list_id)
        .is_none_or(|existing_ref| existing_ref != selection_ref)
    {
        commands
            .entity(sub_refs.list_id)
            .insert(Selection::reference(selection_ref));
    }

    let Ok(refs) = pblr.get(sub_refs.button_id) else {
        return;
    };
    let Ok(mut text) = labels.get_mut(refs.label()) else {
        return;
    };
    let label = if selection.is_empty() {
        placeholders
            .get(combo_ref)
            .map_or_else(|_| PLACEHOLDER_TEXT.to_string(), |p| p.0.clone())
    } else {
        selected_text(selection, &items)
    };
    text.0 = label;
}

pub(crate) fn selected_text(selection: &SelectionValue, items: &Query<&Item<String>>) -> String {
    selection
        .iter()
        .map(|item| items.get(item).map_or("", |item| item.data.as_str()))
        .join(", ")
}

/// Must run after Bevy resets the node to a button.
fn sync_accessibility(
    combos: Query<(Entity, &ComboRefs), With<Combo>>,
    selections: SelectionParam,
    items: Query<&Item<String>>,
    mut nodes: Query<&mut AccessibilityNode>,
) {
    for (combo, sub_refs) in &combos {
        let Ok(mut node) = nodes.get_mut(sub_refs.button_id) else {
            continue;
        };
        let value = selections
            .get(combo)
            .map_or_else(String::new, |selection| selected_text(selection, &items));
        popup::sync_combo_box(&mut node, &value, sub_refs.list_id);
    }
}

/// Closes an open combo on a primary release outside it. The list counts as inside for a
/// selection of many items.
fn close_combos_on_outside_click(
    mut inputs: MessageReader<PointerInput>,
    pointers: PopupPointers,
    combos: Query<(Entity, &ComboState, &ComboRefs), With<Combo>>,
    selections: SelectionParam,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for pointer in popup::primary_releases(&mut inputs) {
        if pointers.panned(pointer) {
            continue;
        }
        for (combo_ref, &combo_state, subs) in combos.iter() {
            let Some(selection_value) = selections.get(combo_ref) else {
                continue;
            };
            let Ok((mut background_color, mut border_color)) =
                combo_buttons.get_mut(subs.button_id)
            else {
                continue;
            };
            let over_list = pointers.over(Some(pointer), subs.list_id);
            if popup::closes_on_outside_release(
                combo_state == ComboState::Open,
                pointers.over(Some(pointer), subs.button_id),
                (selection_value.allow_multiple() && over_list)
                    || pointers.over_scroll_bar(Some(pointer), subs.list_id),
            ) {
                popup::apply_look(
                    ButtonLook::Resting,
                    &theme,
                    &mut background_color,
                    &mut border_color,
                );
                commands.entity(combo_ref).insert(ComboState::Closed);
            }
        }
    }
}

fn handle_combo_button_mouse_events(
    trigger: On<MouseEvent>,
    combos: Query<(&ComboRefs, &ComboState), With<Combo>>,
    mut combo_buttons: Query<(&mut BackgroundColor, &mut BorderColor), With<ComboButton>>,
    theme: Res<Theme>,
) {
    let Ok((ComboRefs { button_id, .. }, combo_state)) = combos.get(trigger.event().event_target())
    else {
        return;
    };
    let Ok((mut background_color, mut border_color)) = combo_buttons.get_mut(*button_id) else {
        return;
    };

    if let Some(look) = popup::button_look(
        trigger.event().kind,
        trigger.event().hovered(),
        *combo_state == ComboState::Open,
    ) {
        popup::apply_look(look, &theme, &mut background_color, &mut border_color);
    }
}
