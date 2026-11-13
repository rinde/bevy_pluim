//! `Gallery`: a row of image cards over a [`Selection`](crate::select::Selection), in titled
//! groups, exactly one of them picked.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::ecs::entity::EntityHashSet;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::{Activate, Button};

use crate::divider::Divider;
use crate::focus::{
    AccessibilitySync, RovingGroup, RovingItem, TabStop, sync_label, sync_role, sync_toggled,
};
use crate::icon::Icon;
use crate::label::Label;
use crate::select::{Item, SelectionParam, StructureChanges, plan_views};
use crate::theme::{Theme, ThemeColors};

/// The width of a card, in logical pixels.
const CARD_WIDTH: f32 = 104.0;
/// The height of a card.
const CARD_HEIGHT: f32 = 92.0;
/// The width of the image of a card.
const IMAGE_WIDTH: f32 = 72.0;
/// The height of the image of a card.
const IMAGE_HEIGHT: f32 = 30.0;
/// The gap between the image and the label of a card.
const CARD_GAP: f32 = 6.0;
/// The padding of a card, inside its border.
const CARD_PADDING: f32 = 6.0;
/// The gap between two cards of a group.
const CARDS_GAP: f32 = 6.0;
/// The gap between a group heading and its cards.
const HEADING_GAP: f32 = 8.0;
/// The gap between two groups and the divider between them.
const GROUP_GAP: f32 = 14.0;
/// The width of the ring of the picked card.
const RING_WIDTH: f32 = 2.0;

/// Registers the systems and observers of [`Gallery`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Copy, Clone, Debug)]
pub struct GalleryPlugin;

impl Plugin for GalleryPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Item<GalleryCard>>()
            .register_type::<GalleryCard>()
            .register_type::<Gallery>()
            .register_type::<GalleryCardView>()
            .add_systems(Update, (sync_cards, update_cards).chain())
            .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
            .add_observer(on_activate);
    }
}

/// A row of image cards over a [`Selection`](crate::select::Selection) of
/// [`Item<GalleryCard>`]s, under titled groups, where picking a card selects its item.
///
/// Put it on an entity together with a selection that has [`GalleryCard`] items, such as
/// `Selection::builder().one().build_with_items([Item { data: GalleryCard::new(image, "Steam", "Locomotives") }])`,
/// or with a [`Selection::Ref`](crate::select::Selection::Ref) to one. The selection should be a
/// `One`: exactly one card is picked across all groups, and picking another deselects the
/// first.
///
/// Neighbouring items with the same [`GalleryCard::group`] form a group: an overline
/// [`Label`] with the title above a row of cards. Groups sit side by side with a vertical
/// [`Divider`] between them, and do not shrink, so a gallery wider than its parent overflows:
/// put it in a horizontal [`ScrollArea`](crate::scroll_area::ScrollArea) to scroll it.
///
/// A card is a [`Button`]: Enter and Space on a focused one pick it, like a click. As a whole the gallery is one tab stop, at its selected item (the first when none is selected).
/// Left and up move the focus to the previous item and right and down to the next, wrapping in
/// the order shown, through the groups; Home and End go to the first and last. For a selection of at most one item
/// the arrows also pick the item they land on (never toggling it off); for a selection of many
/// they move the focus only, and Enter or Space toggle. Disabled and hidden items are skipped.
///
/// The gallery owns only the groups and dividers it spawns, and replaces just those when its
/// items or groups change. Children the application puts under the gallery entity are never
/// despawned by a rebuild. Those present before the first build stay before the gallery's own
/// children; a child added later is not reordered, and a rebuild appends the new groups
/// after it.
///
/// To a screen reader the gallery is a radio group and each card a radio button named after
/// its label, checked while it is picked. The library gives the group no label; an app can
/// insert Bevy's `AccessibleLabel` on the gallery.
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("Gallery"),
    Node {
        align_items: AlignItems::FlexStart,
        column_gap: Val::Px(GROUP_GAP),
        flex_shrink: 0.0,
        ..default()
    },
    RovingGroup = RovingGroup::radio(),
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::RadioGroup)),
)]
pub struct Gallery;

/// What a card of a [`Gallery`] shows: an image, a label and the group it is under.
#[derive(Debug, Clone, PartialEq, Eq, Reflect)]
pub struct GalleryCard {
    /// The image, white on transparent (see [`Icon`]), so that it follows the theme.
    pub image: Handle<Image>,
    /// The text under the image, which wraps to a second line; also the accessible name of
    /// the card.
    pub label: String,
    /// The title of the group the card is under.
    pub group: String,
}

impl GalleryCard {
    /// A card showing `image` and `label` in the group titled `group`.
    pub fn new(image: Handle<Image>, label: impl Into<String>, group: impl Into<String>) -> Self {
        Self {
            image,
            label: label.into(),
            group: group.into(),
        }
    }
}

/// One card of a [`Gallery`]: picks its item when it is activated.
///
/// Only the widget builds one, and the component is immutable: the item and the gallery are
/// fixed for the life of the card and are read with [`item`](Self::item) and
/// [`owner`](Self::owner). A card's accessible name is its [`GalleryCard::label`], so callers
/// supply meaningful labels.
///
/// ```compile_fail
/// use bevy_pluim::gallery::GalleryCardView;
/// # let (item, gallery) = (bevy::prelude::Entity::PLACEHOLDER, bevy::prelude::Entity::PLACEHOLDER);
/// let _ = GalleryCardView { item, owner: gallery };
/// ```
///
/// ```compile_fail
/// use bevy::prelude::*;
/// use bevy_pluim::gallery::GalleryCardView;
/// fn change(mut cards: Query<&mut GalleryCardView>) {}
/// bevy::ecs::system::assert_is_system(change);
/// ```
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
#[component(immutable)]
pub struct GalleryCardView {
    item: Entity,
    owner: Entity,
}

impl GalleryCardView {
    pub(crate) const fn new(item: Entity, owner: Entity) -> Self {
        Self { item, owner }
    }

    /// The [`Item<GalleryCard>`] this card shows.
    #[must_use]
    pub const fn item(&self) -> Entity {
        self.item
    }

    /// The [`Gallery`] this card belongs to.
    #[must_use]
    pub const fn owner(&self) -> Entity {
        self.owner
    }
}

/// The children of a [`GalleryCardView`].
#[derive(Component, Debug, Copy, Clone, Reflect)]
#[reflect(Component)]
pub struct GalleryCardRefs {
    /// The [`Icon`] showing the image.
    pub image: Entity,
    /// The label text.
    pub label: Entity,
}

/// The groups a [`Gallery`] was built for, as titles with their items, to tell whether the
/// children still match the items.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
struct BuiltGroups(Vec<(String, Vec<Entity>)>);

/// Marks the groups and dividers the gallery spawned itself, so a rebuild replaces those and
/// leaves children the application put under the gallery alone.
#[derive(Component, Debug, Copy, Clone)]
struct GalleryOwned;

fn label_font(theme: &Theme) -> TextFont {
    TextFont {
        font: theme.body_font.clone().into(),
        font_size: Theme::font_size(theme.font_size_small),
        weight: FontWeight::SEMIBOLD,
        ..default()
    }
}

/// Builds the groups and dividers of every [`Gallery`] again when its items or their groups no
/// longer match what was built, moving the cards of items that stay into them.
fn sync_cards(
    mut changes: StructureChanges<Gallery, GalleryCard>,
    galleries: Query<(Option<&Children>, Option<&BuiltGroups>), With<Gallery>>,
    owned: Query<(), With<GalleryOwned>>,
    nodes: Query<&Children>,
    cards: Query<&GalleryCardView>,
    items: Query<&Item<GalleryCard>>,
    selections: SelectionParam,
    theme: Res<Theme>,
    mut commands: Commands,
) {
    for gallery in changes.affected(&selections) {
        let Ok((children, built)) = galleries.get(gallery) else {
            continue;
        };
        let Some((_, values)) = selections.get_items(gallery) else {
            continue;
        };
        let mut wanted: Vec<(String, Vec<Entity>)> = Vec::new();
        for item in values.iter().copied() {
            let Ok(card) = items.get(item) else {
                continue;
            };
            match wanted.last_mut() {
                Some((title, cards)) if *title == card.data.group => cards.push(item),
                _ => wanted.push((card.data.group.clone(), vec![item])),
            }
        }
        let wanted = BuiltGroups(wanted);
        if built == Some(&wanted) {
            continue;
        }
        // The groups, headings and dividers are cheap and are made again, but a card of an item
        // that stays is moved into its new row as the same entity, so its focus, hover and
        // children survive a change elsewhere in the gallery.
        let old = children
            .into_iter()
            .flat_map(RelationshipTarget::iter)
            .filter(|child| owned.contains(*child))
            .collect::<Vec<_>>();
        let plan = plan_views(
            old.iter()
                .flat_map(|group| nodes.iter_descendants(*group))
                .filter_map(|card| Some((card, Some(cards.get(card).ok()?.item())))),
            &wanted
                .0
                .iter()
                .flat_map(|(_, items)| items.iter().copied())
                .collect::<Vec<_>>(),
        );
        let mut kept = plan.slots.into_iter();
        for (index, (title, group_cards)) in wanted.0.iter().enumerate() {
            if index > 0 {
                commands.spawn((Divider::vertical(), GalleryOwned, ChildOf(gallery)));
            }
            let group = commands
                .spawn((
                    Name::new("GalleryGroup"),
                    GalleryOwned,
                    Node {
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Px(HEADING_GAP),
                        flex_shrink: 0.0,
                        ..default()
                    },
                    ChildOf(gallery),
                ))
                .id();
            commands.spawn((Label::overline(title.as_str()), ChildOf(group)));
            let row = commands
                .spawn((
                    Name::new("GalleryCards"),
                    Node {
                        column_gap: Val::Px(CARDS_GAP),
                        ..default()
                    },
                    ChildOf(group),
                ))
                .id();
            for item in group_cards {
                match kept.next().flatten() {
                    Some(card) => {
                        commands.entity(card).insert(ChildOf(row));
                    }
                    None => spawn_card(&mut commands, &theme, gallery, row, *item),
                }
            }
        }
        // After the kept cards left, which takes the stale ones along with their old groups.
        for child in old {
            commands.entity(child).despawn();
        }
        commands.entity(gallery).insert(wanted);
    }
}

fn spawn_card(commands: &mut Commands, theme: &Theme, gallery: Entity, row: Entity, item: Entity) {
    // The children are spawned under the card's reserved entity, so no node is ever parentless.
    let card = commands.spawn_empty().id();
    let image = commands
        .spawn((
            Icon::bundle(Handle::default(), IMAGE_HEIGHT),
            Pickable::IGNORE,
            ChildOf(card),
        ))
        // The image is wider than tall, which `Icon::bundle`'s square is not.
        .insert(Node {
            width: Val::Px(IMAGE_WIDTH),
            height: Val::Px(IMAGE_HEIGHT),
            flex_shrink: 0.0,
            ..default()
        })
        .id();
    let label = commands
        .spawn((
            Text::new(""),
            TextLayout::justify(Justify::Center),
            label_font(theme),
            Node {
                max_width: Val::Percent(100.0),
                ..default()
            },
            Pickable::IGNORE,
            ChildOf(card),
        ))
        .id();
    commands.entity(card).insert((
        Name::new("GalleryCard"),
        GalleryCardView::new(item, gallery),
        RovingItem::new(gallery, item),
        GalleryCardRefs { image, label },
        Button,
        Hovered::default(),
        TabStop,
        Node {
            width: Val::Px(CARD_WIDTH),
            height: Val::Px(CARD_HEIGHT),
            padding: UiRect::all(Val::Px(CARD_PADDING)),
            border: UiRect::all(theme.border_thickness),
            border_radius: BorderRadius::all(theme.control_radius),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: Val::Px(CARD_GAP),
            flex_shrink: 0.0,
            ..default()
        },
        BackgroundColor(theme.colors().card_surface),
        BorderColor::all(theme.colors().border),
        ChildOf(row),
    ));
}

/// How a card looks.
#[derive(Debug, Copy, Clone, PartialEq)]
struct CardLook {
    background: Color,
    border: Color,
    ring: bool,
}

/// The look of a card from its state: picked, hovered or at rest. Used by the system that
/// restyles cards on every change, so a theme change under a resting pointer keeps the hover.
const fn card_look(colors: &ThemeColors, picked: bool, hovered: bool) -> CardLook {
    if picked {
        CardLook {
            background: colors.hovered_selected_item,
            border: colors.selection_ring,
            ring: true,
        }
    } else {
        CardLook {
            background: if hovered {
                colors.hovered_item
            } else {
                colors.card_surface
            },
            border: colors.border,
            ring: false,
        }
    }
}

/// Puts the image, label, colours and fonts of every card in step with its item, its
/// selection, the pointer and the theme.
fn update_cards(
    theme: Res<Theme>,
    selections: SelectionParam,
    items: Query<&Item<GalleryCard>>,
    mut cards: Query<(
        &GalleryCardView,
        Ref<GalleryCardRefs>,
        &Hovered,
        &mut BackgroundColor,
        &mut BorderColor,
        &mut Node,
    )>,
    mut images: Query<&mut ImageNode, Without<GalleryCardView>>,
    mut labels: Query<(&mut Text, &mut TextColor, &mut TextFont), Without<GalleryCardView>>,
) {
    for (view, refs, hovered, mut background, mut border, mut node) in &mut cards {
        let Ok(item) = items.get(view.item) else {
            continue;
        };
        let picked = selections
            .get(view.owner())
            .is_some_and(|selection| selection.is_selected(view.item));
        let look = card_look(theme.colors(), picked, hovered.get());
        background.set_if_neq(BackgroundColor(look.background));
        border.set_if_neq(BorderColor::all(look.border));
        let thickness = if look.ring {
            Val::Px(RING_WIDTH)
        } else {
            theme.border_thickness
        };
        if node.border.left != thickness {
            node.border = UiRect::all(thickness);
        }
        if theme.is_changed() {
            node.border_radius = BorderRadius::all(theme.control_radius);
        }

        if let Ok(mut image) = images.get_mut(refs.image)
            && image.image != item.data.image
        {
            image.image = item.data.image.clone();
        }
        if let Ok((mut text, mut color, mut font)) = labels.get_mut(refs.label) {
            if text.0 != item.data.label {
                item.data.label.clone_into(&mut text.0);
            }
            color.set_if_neq(TextColor(if picked || hovered.get() {
                theme.colors().text_primary
            } else {
                theme.colors().text_secondary
            }));
            if theme.is_changed() || refs.is_added() {
                *font = label_font(&theme);
            }
        }
    }
}

/// Keeps the gallery a radio group and each card a radio button named after the label of its
/// item, toggled while it is picked. Bevy makes a node a button again when its `Button`
/// changes, and clears the label of a button that has no text of its own when it is added, so
/// this runs after it.
fn sync_accessibility(
    mut galleries: Query<
        &mut AccessibilityNode,
        (
            With<Gallery>,
            Without<GalleryCardView>,
            Changed<AccessibilityNode>,
        ),
    >,
    mut cards: ParamSet<(
        Query<
            (&GalleryCardView, &mut AccessibilityNode),
            Or<(Changed<AccessibilityNode>, Changed<GalleryCardView>)>,
        >,
        Query<(&GalleryCardView, &mut AccessibilityNode)>,
    )>,
    items: Query<&Item<GalleryCard>>,
    selections: SelectionParam,
    mut changes: StructureChanges<Gallery, GalleryCard>,
) {
    // The container is not a `Button`, so nothing of Bevy's resets its role; it is set once by
    // `#[require]` and re-synced only when its node changes (or is added).
    for mut node in &mut galleries {
        sync_role(&mut node, Role::RadioGroup);
    }
    // A card is looked at when Bevy reset its node or it is new, and (for its label and
    // toggled state) when its gallery's items or selection changed. A quiet frame does neither.
    for (view, mut node) in &mut cards.p0() {
        sync_card(view, &mut node, &items, &selections);
    }
    let affected: EntityHashSet = changes.affected(&selections).into_iter().collect();
    if affected.is_empty() {
        return;
    }
    for (view, mut node) in &mut cards.p1() {
        if affected.contains(&view.owner()) {
            sync_card(view, &mut node, &items, &selections);
        }
    }
}

/// Writes the role, label and toggled state of one card's node, only where they differ.
fn sync_card(
    view: &GalleryCardView,
    node: &mut Mut<AccessibilityNode>,
    items: &Query<&Item<GalleryCard>>,
    selections: &SelectionParam,
) {
    let Ok(item) = items.get(view.item) else {
        return;
    };
    let picked = selections
        .get(view.owner())
        .is_some_and(|selection| selection.is_selected(view.item));
    sync_role(node, Role::RadioButton);
    sync_label(node, &item.data.label);
    sync_toggled(node, Some(picked));
}

/// Picks the card that was used (a click, or Enter or Space on it).
fn on_activate(t: On<Activate>, views: Query<&GalleryCardView>, mut selections: SelectionParam) {
    let Ok(view) = views.get(t.event().event_target()) else {
        return;
    };
    selections.update_selection(view.owner(), |selection| {
        selection.toggle_selected(view.item)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{DEFAULT_DARK_THEME_COLORS, DEFAULT_LIGHT_THEME_COLORS};

    #[test]
    fn the_picked_card_has_the_selection_ring_and_the_strong_hover_background() {
        for colors in [DEFAULT_LIGHT_THEME_COLORS, DEFAULT_DARK_THEME_COLORS] {
            let look = card_look(&colors, true, false);
            assert_eq!(look.background, colors.hovered_selected_item);
            assert_eq!(look.border, colors.selection_ring);
            assert!(look.ring);
        }
    }

    #[test]
    fn a_card_that_is_not_picked_is_a_surface_and_hovers_to_the_hover_background() {
        let colors = DEFAULT_LIGHT_THEME_COLORS;
        let rest = card_look(&colors, false, false);
        assert_eq!(rest.background, colors.card_surface);
        assert_eq!(rest.border, colors.border);
        assert!(!rest.ring);
        assert_eq!(
            card_look(&colors, false, true).background,
            colors.hovered_item
        );
    }
}
