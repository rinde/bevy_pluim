//! `LightDarkToggle`: a three-way light / system / dark switch that needs no configuration.

use accesskit::Role;
use bevy::a11y::AccessibilityNode;
use bevy::asset::RenderAssetUsages;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use bevy::ui::Selected;
use bevy::ui_widgets::{Activate, Button};
use bevy::window::{PrimaryWindow, WindowTheme, WindowThemeChanged};

use crate::focus::{
    AccessibilitySync, RovingGroup, RovingItem, TabStop, sync_label, sync_role, sync_toggled,
};
use crate::glyph_images::{GlyphKey, shared};
use crate::indicator::{self, Indicator};
use crate::list::sync_selected;
use crate::segmented_control::segment_colors;
use crate::theme::{Theme, ThemeMode};

/// The height of an option.
const OPTION_HEIGHT: f32 = 28.0;
/// The padding between an option's content and its sides.
const OPTION_SIDE_PADDING: f32 = 10.0;
/// The padding of the track around its options.
const TRACK_PADDING: f32 = 3.0;
/// The gap between two options.
const OPTION_GAP: f32 = 2.0;
/// The gap between an option's icon and its label.
const ICON_GAP: f32 = 6.0;
/// The size of an option's icon.
const ICON_SIZE: u32 = 16;
/// The size of an option's label.
const OPTION_FONT_SIZE: f32 = 13.0;

/// Registers the systems and observers of [`LightDarkToggle`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct LightDarkTogglePlugin;

impl Plugin for LightDarkTogglePlugin {
    fn build(&self, app: &mut App) {
        // `WindowPlugin` spawns the primary window while it builds, so the window of a normal app
        // exists before this plugin does and no `Add<Window>` observer will see it.
        let preset: Vec<Entity> = app
            .world_mut()
            .query::<(Entity, &Window)>()
            .iter(app.world())
            .filter(|(_, window)| window.window_theme.is_some())
            .map(|(entity, _)| entity)
            .collect();
        for entity in preset {
            app.world_mut().entity_mut(entity).insert(PresetWindowTheme);
        }
        app.init_resource::<SystemTheme>()
            .add_observer(mark_preset_window_theme)
            .add_message::<WindowThemeChanged>()
            .add_systems(PreUpdate, track_system_theme)
            .add_systems(
                Update,
                (
                    apply_preference
                        .run_if(resource_exists::<ThemePreference>)
                        .run_if(preference_or_system_changed),
                    apply_window_theme.run_if(resource_exists::<ThemePreference>),
                    (|| None::<Entity>)
                        .pipe(theme_toggles)
                        .run_if(theme_toggles_stale),
                    indicator::spawn::<OptionIndicator, LightDarkToggle>,
                    indicator::animate::<OptionIndicator, LightDarkToggle, LightDarkOption>,
                )
                    .chain(),
            )
            .add_systems(PostUpdate, sync_accessibility.in_set(AccessibilitySync))
            .register_type::<LightDarkToggle>()
            .register_type::<LightDarkOption>()
            .register_type::<ThemePreference>()
            .register_type::<SystemTheme>();
    }
}

/// Which colours the app wants: its [`Theme`] follows this.
///
/// A [`LightDarkToggle`] sets it, and adding the first toggle inserts the default,
/// [`System`](Self::System), so the app follows the system's light or dark setting. An app that
/// wants light or dark fixed inserts `ThemePreference::Light` or `ThemePreference::Dark` (before
/// or after adding the toggle; an existing value is kept). Without this resource nothing
/// touches the [`Theme`]'s mode or the window.
#[derive(Resource, Debug, Copy, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Resource)]
pub enum ThemePreference {
    /// Always the light colours.
    Light,
    /// The light or dark colours as the system says (light when it is not known).
    #[default]
    System,
    /// Always the dark colours.
    Dark,
}

impl ThemePreference {
    /// The label of its option in a [`LightDarkToggle`].
    const fn label(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::System => "System",
            Self::Dark => "Dark",
        }
    }
}

/// The light or dark setting of the system, as far as the app has heard of it.
///
/// Read from the primary window when the app starts, and kept up to date from Bevy's
/// [`WindowThemeChanged`] messages, which the windowing system sends when the person changes
/// the setting. `None` when no window has told (headless, or a system without the setting).
/// A primary window spawned with a fixed [`Window::window_theme`] says nothing about the system:
/// the windowing system reports that theme back as the window's, so it is not read, and
/// `SystemTheme` stays `None` until a [`WindowThemeChanged`] arrives.
/// While the preference is fixed to light or dark the window is told to use that theme, and
/// some platforms then stop reporting the system's, so a change made in that time may only be
/// seen when the app goes back to [`ThemePreference::System`].
#[derive(Resource, Debug, Copy, Clone, Default, PartialEq, Eq, Reflect)]
#[reflect(Resource)]
pub struct SystemTheme(pub Option<ThemeMode>);

const fn mode_of(theme: WindowTheme) -> ThemeMode {
    match theme {
        WindowTheme::Light => ThemeMode::Light,
        WindowTheme::Dark => ThemeMode::Dark,
    }
}

/// Marks a window that was spawned with a fixed theme, whose `window_theme` is then not the
/// system's.
#[derive(Component, Debug, Copy, Clone)]
struct PresetWindowTheme;

/// Marks a window spawned with a [`Window::window_theme`], before the windowing system fills it
/// in with the theme it really uses.
fn mark_preset_window_theme(add: On<Add<Window>>, windows: Query<&Window>, mut commands: Commands) {
    if windows
        .get(add.entity)
        .is_ok_and(|w| w.window_theme.is_some())
    {
        commands.entity(add.entity).insert(PresetWindowTheme);
    }
}

/// Keeps [`SystemTheme`] up to date.
fn track_system_theme(
    mut changes: MessageReader<WindowThemeChanged>,
    windows: Query<(&Window, Has<PresetWindowTheme>), With<PrimaryWindow>>,
    mut system: ResMut<SystemTheme>,
    mut probed: Local<bool>,
) {
    // The window's own theme is the system's until we fix it, which we only do later, unless
    // the app fixed it at creation.
    if !*probed && let Some((window, preset)) = windows.iter().next() {
        *probed = true;
        if !preset && let Some(theme) = window.window_theme {
            system.set_if_neq(SystemTheme(Some(mode_of(theme))));
        }
    }
    for change in changes.read() {
        system.set_if_neq(SystemTheme(Some(mode_of(change.theme))));
    }
}

/// Puts the [`Theme`] in the mode the preference and the system ask for.
fn apply_preference(
    preference: Res<ThemePreference>,
    system: Res<SystemTheme>,
    mut theme: ResMut<Theme>,
) {
    let mode = match *preference {
        ThemePreference::Light => ThemeMode::Light,
        ThemePreference::Dark => ThemeMode::Dark,
        ThemePreference::System => system.0.unwrap_or(ThemeMode::Light),
    };
    if theme.mode() != mode {
        theme.set_mode(mode);
    }
}

/// Makes the primary window adopt the preference: light or dark when fixed, the system's own
/// when following it.
fn apply_window_theme(
    preference: Res<ThemePreference>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let wanted = match *preference {
        ThemePreference::Light => Some(WindowTheme::Light),
        ThemePreference::Dark => Some(WindowTheme::Dark),
        ThemePreference::System => None,
    };
    for mut window in &mut windows {
        if window.window_theme != wanted {
            window.window_theme = wanted;
        }
    }
}

/// A switch between light, the system's setting and dark, with an icon and a label on each.
///
/// Spawn it and nothing else: it sets the [`ThemePreference`] (starting at
/// [`System`](ThemePreference::System) unless the app inserted one), the preference sets the
/// [`Theme`]'s mode, and the primary window, if there is one, adopts light or dark with it.
/// The icons (sun, computer and moon) are drawn by the library. To a screen reader the toggle
/// is a radio group named "Theme" and each option a radio button, toggled while it is the
/// preference; the options are one tab stop: the arrow keys, Home and End move between them and pick the option
/// they land on, and Enter or Space picks the focused one.
#[derive(Component, Debug, Copy, Clone, Default, Reflect)]
#[reflect(Component)]
#[component(on_add = Self::on_add)]
#[require(
    Name::new("LightDarkToggle"),
    Node {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        align_self: AlignSelf::FlexStart,
        padding: UiRect::all(Val::Px(TRACK_PADDING)),
        column_gap: Val::Px(OPTION_GAP),
        ..default()
    },
    BackgroundColor::default(),
    AccessibilityNode = AccessibilityNode::from(accesskit::Node::new(Role::RadioGroup)),
    RovingGroup = RovingGroup::marked_radio(),
)]
pub struct LightDarkToggle;

/// One option of a [`LightDarkToggle`]: a button that sets the [`ThemePreference`].
#[derive(Component, Debug, Copy, Clone, PartialEq, Eq, Reflect)]
#[reflect(Component)]
#[require(
    Name::new("LightDarkOption"),
    Button,
    Hovered,
    TabStop,
    BackgroundColor::default()
)]
pub struct LightDarkOption {
    /// What this option asks for.
    pub preference: ThemePreference,
}

/// The highlight behind the selected option of a [`LightDarkToggle`]: a child of the toggle,
/// placed absolutely behind the options, which slides to the selected option when the preference
/// changes.
#[derive(Component, Debug, Copy, Clone)]
#[require(Name::new("OptionIndicator"), Node, BackgroundColor::default())]
struct OptionIndicator {
    owner: Entity,
}

impl Indicator for OptionIndicator {
    fn of(owner: Entity) -> Self {
        Self { owner }
    }

    fn owner(&self) -> Entity {
        self.owner
    }
}

/// Marks the icon of an option.
#[derive(Component, Debug, Copy, Clone)]
struct OptionIcon;

/// Marks the label of an option.
#[derive(Component, Debug, Copy, Clone)]
struct OptionLabel;

impl LightDarkToggle {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        if world.get_resource::<ThemePreference>().is_none() {
            world.commands().insert_resource(ThemePreference::default());
        }
        let current = world
            .get_resource::<ThemePreference>()
            .copied()
            .unwrap_or_default();
        let icons = if world.contains_resource::<Assets<Image>>() {
            [
                Some(shared(&mut world, GlyphKey::Sun, || icon_image(Glyph::Sun))),
                Some(shared(&mut world, GlyphKey::Computer, || {
                    icon_image(Glyph::Computer)
                })),
                Some(shared(&mut world, GlyphKey::Moon, || {
                    icon_image(Glyph::Moon)
                })),
            ]
        } else {
            [None, None, None]
        };
        let toggle = context.entity;
        let mut commands = world.commands();
        for (preference, icon) in [
            ThemePreference::Light,
            ThemePreference::System,
            ThemePreference::Dark,
        ]
        .into_iter()
        .zip(icons)
        {
            // The option is its own roving item: the toggle's state is the preference, not a
            // selection of items.
            let option_id = commands.spawn_empty().id();
            let mut option = commands.entity(option_id);
            option.insert((
                LightDarkOption { preference },
                RovingItem::new(toggle, option_id),
                Node::default(),
                ChildOf(toggle),
            ));
            option
                .observe(
                    move |_: On<Activate>, mut preference_res: ResMut<ThemePreference>| {
                        preference_res.set_if_neq(preference);
                    },
                )
                .with_children(|parent| {
                    parent.spawn((
                        OptionIcon,
                        ImageNode::new(icon.unwrap_or_default()),
                        Node {
                            width: Val::Px(ICON_SIZE as f32),
                            height: Val::Px(ICON_SIZE as f32),
                            margin: UiRect::right(Val::Px(ICON_GAP)),
                            ..default()
                        },
                        Pickable::IGNORE,
                    ));
                    parent.spawn((Text::new(preference.label()), OptionLabel, Pickable::IGNORE));
                });
            if preference == current {
                option.insert(Selected);
            }
        }
        commands.run_system_cached_with(theme_toggles, Some(toggle));
    }
}

#[derive(Clone, Copy)]
enum Glyph {
    Sun,
    Computer,
    Moon,
}

/// The distance from a point to a segment.
fn segment_distance(p: Vec2, a: Vec2, b: Vec2) -> f32 {
    let ab = b - a;
    let t = ((p - a).dot(ab) / ab.length_squared()).clamp(0.0, 1.0);
    p.distance(a + ab * t)
}

/// The signed distance from a point to a box of half size `half` around the origin.
fn box_distance(p: Vec2, half: Vec2) -> f32 {
    let q = p.abs() - half;
    q.max(Vec2::ZERO).length() + q.x.max(q.y).min(0.0)
}

/// The signed distance of a point of the 16 px icon to the shape: negative inside.
fn shape_distance(icon: Glyph, p: Vec2) -> f32 {
    let centre = Vec2::splat(ICON_SIZE as f32 / 2.0);
    match icon {
        Glyph::Sun => {
            let disc = p.distance(centre) - 3.2;
            (0..8).fold(disc, |d, i| {
                let direction = Vec2::from_angle(i as f32 * std::f32::consts::FRAC_PI_4);
                let ray =
                    segment_distance(p, centre + direction * 5.2, centre + direction * 7.0) - 0.7;
                d.min(ray)
            })
        }
        Glyph::Moon => {
            let disc = p.distance(centre) - 6.0;
            let bite = p.distance(centre + Vec2::new(3.2, -2.6)) - 5.0;
            disc.max(-bite)
        }
        Glyph::Computer => {
            let screen = box_distance(p - Vec2::new(8.0, 6.5), Vec2::new(6.5, 4.5)).abs() - 0.75;
            let stem = box_distance(p - Vec2::new(8.0, 12.0), Vec2::new(0.9, 1.5));
            let base = box_distance(p - Vec2::new(8.0, 13.6), Vec2::new(3.8, 0.8));
            screen.min(stem).min(base)
        }
    }
}

/// A white-on-transparent, antialiased icon.
fn icon_image(icon: Glyph) -> Image {
    let mut data = vec![0u8; (ICON_SIZE * ICON_SIZE * 4) as usize];
    for y in 0..ICON_SIZE {
        for x in 0..ICON_SIZE {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let coverage = (0.5 - shape_distance(icon, p)).clamp(0.0, 1.0);
            let at = ((y * ICON_SIZE + x) * 4) as usize;
            if let Some(pixel) = data.get_mut(at..at + 4) {
                #[expect(
                    clippy::cast_sign_loss,
                    reason = "coverage is clamped to 0..=1, so the alpha is 0..=255"
                )]
                let alpha = (coverage * 255.0).round() as u8;
                pixel.copy_from_slice(&[255, 255, 255, alpha]);
            }
        }
    }
    Image::new(
        Extent3d {
            width: ICON_SIZE,
            height: ICON_SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    )
}

/// Whether the preference or the system theme changed.
fn preference_or_system_changed(
    preference: Option<Res<ThemePreference>>,
    system: Res<SystemTheme>,
) -> bool {
    system.is_changed() || preference.is_some_and(|preference| preference.is_changed())
}

/// Whether the toggles' look may be out of date.
fn theme_toggles_stale(
    theme: Res<Theme>,
    preference: Option<Res<ThemePreference>>,
    hovered: Query<(), (With<LightDarkOption>, Changed<Hovered>)>,
) -> bool {
    theme.is_changed()
        || preference.is_some_and(|preference| preference.is_changed())
        || !hovered.is_empty()
}

fn theme_toggles(
    toggle_id: In<Option<Entity>>,
    theme: Res<Theme>,
    preference: Option<Res<ThemePreference>>,
    mut toggles: Query<
        (Entity, &mut Node, &mut BackgroundColor),
        (
            With<LightDarkToggle>,
            Without<LightDarkOption>,
            Without<OptionIndicator>,
        ),
    >,
    mut options: Query<
        (
            Entity,
            &ChildOf,
            &LightDarkOption,
            &Hovered,
            &mut Node,
            &mut BackgroundColor,
        ),
        (Without<LightDarkToggle>, Without<OptionIndicator>),
    >,
    mut indicators: Query<
        (&OptionIndicator, &mut Node, &mut BackgroundColor),
        (Without<LightDarkToggle>, Without<LightDarkOption>),
    >,
    mut icons: Query<(&ChildOf, &mut ImageNode), With<OptionIcon>>,
    mut labels: Query<(&ChildOf, &mut TextColor, &mut TextFont), With<OptionLabel>>,
    selected: Query<(), With<Selected>>,
    mut commands: Commands,
) {
    for (toggle, mut node, mut background) in &mut toggles {
        if toggle_id.0.is_some_and(|id| id != toggle) {
            continue;
        }
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = theme.colors().segmented_track;
    }

    let preference = preference.map(|p| *p);
    // Each option's colours: (selected, hovered), by the option's entity.
    let mut colours = Vec::new();
    for (option_id, child_of, option, hovered, mut node, mut background) in &mut options {
        if toggle_id.0.is_some_and(|id| id != child_of.parent()) {
            continue;
        }
        let is_selected = preference == Some(option.preference);
        if is_selected != selected.contains(option_id) {
            sync_selected(&mut commands, option_id, !is_selected, is_selected);
        }
        node.height = Val::Px(OPTION_HEIGHT);
        node.padding = UiRect::axes(Val::Px(OPTION_SIDE_PADDING), Val::ZERO);
        node.align_items = AlignItems::Center;
        node.justify_content = JustifyContent::Center;
        node.border_radius = BorderRadius::all(theme.control_radius);
        // The selected look is the indicator's, which slides.
        background.0 = Color::NONE;
        colours.push((
            option_id,
            segment_colors(theme.colors(), is_selected, hovered.get()).1,
        ));
    }

    for (indicator, mut node, mut background) in &mut indicators {
        if toggle_id.0.is_some_and(|id| id != indicator.owner) {
            continue;
        }
        node.border_radius = BorderRadius::all(theme.control_radius);
        background.0 = OptionIndicator::color(&theme);
    }

    for (child_of, mut image) in &mut icons {
        if let Some((_, ink)) = colours.iter().find(|(id, _)| *id == child_of.parent()) {
            image.color = *ink;
        }
    }
    for (child_of, mut text_color, mut text_font) in &mut labels {
        let Some((_, ink)) = colours.iter().find(|(id, _)| *id == child_of.parent()) else {
            continue;
        };
        text_color.0 = *ink;
        *text_font = TextFont {
            font: theme.body_font.clone().into(),
            font_size: Theme::font_size(OPTION_FONT_SIZE),
            weight: FontWeight::SEMIBOLD,
            ..default()
        };
    }
}

/// Keeps the toggle a radio group named "Theme" and each option a radio button named after its
/// label, toggled while it is the preference. Bevy makes a node a button again when its `Button`
/// changes, so this runs after it.
fn sync_accessibility(
    mut toggles: Query<
        &mut AccessibilityNode,
        (
            With<LightDarkToggle>,
            Without<LightDarkOption>,
            Changed<AccessibilityNode>,
        ),
    >,
    mut options: ParamSet<(
        Query<
            (&LightDarkOption, &mut AccessibilityNode),
            (
                Without<LightDarkToggle>,
                Or<(Changed<AccessibilityNode>, Changed<LightDarkOption>)>,
            ),
        >,
        Query<(&LightDarkOption, &mut AccessibilityNode), Without<LightDarkToggle>>,
    )>,
    preference: Option<Res<ThemePreference>>,
) {
    for mut node in &mut toggles {
        sync_role(&mut node, Role::RadioGroup);
        sync_label(&mut node, "Theme");
    }
    // An option is looked at when Bevy reset its node or it is new, and all of them when the
    // preference changed; an idle frame touches none.
    let current = preference.as_deref().copied();
    for (option, mut node) in &mut options.p0() {
        sync_option(*option, &mut node, current);
    }
    if preference.as_ref().is_some_and(DetectChanges::is_changed) {
        for (option, mut node) in &mut options.p1() {
            sync_option(*option, &mut node, current);
        }
    }
}

/// Writes the label, role and toggled state of one option's node, only where they differ.
fn sync_option(
    option: LightDarkOption,
    node: &mut Mut<AccessibilityNode>,
    current: Option<ThemePreference>,
) {
    sync_label(node, option.preference.label());
    sync_role(node, Role::RadioButton);
    sync_toggled(node, Some(current == Some(option.preference)));
}
