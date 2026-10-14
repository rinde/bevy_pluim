//! The [`Theme`] resource that styles every widget, and its default light and dark colours.

use bevy::color::palettes::tailwind::{
    BLUE_200, BLUE_400, BLUE_500, BLUE_600, BLUE_700, BLUE_800, BLUE_900, GRAY_50, GRAY_100,
    GRAY_200, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900, GRAY_950,
};
use bevy::prelude::*;
use educe::Educe;

/// Registers the systems and observers of the [`Theme`].
///
/// Added by [`WidgetsPlugin`](crate::WidgetsPlugin).
#[derive(Debug, Copy, Clone)]
pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_theme.run_if(resource_changed::<Theme>));
    }
}

fn update_theme(mut clear_color: ResMut<ClearColor>, theme: Res<Theme>) {
    clear_color.0 = theme.colors.background_primary;
}

/// How every widget looks.
///
/// Changing the resource restyles all widgets, and keeps [`ClearColor`] in step with the background.
#[derive(Resource, Clone, Debug, Educe, Reflect)]
#[reflect(Resource)]
#[educe(Default)]
pub struct Theme {
    /// The radius of the corners of controls: buttons, segments, list items, steppers and scroll bars.
    #[educe(Default = Val::Px(8.0))]
    pub control_radius: Val,
    /// The radius of the corners of panels: cards and popups.
    #[educe(Default = Val::Px(12.0))]
    pub panel_radius: Val,
    /// The thickness of borders.
    #[educe(Default = Val::Px(1.0))]
    pub border_thickness: Val,

    /// The colours in use: the light or the dark set, as [`mode`](Self::mode) says.
    ///
    /// Widgets read this one place. Change the mode with [`set_mode`](Self::set_mode) or
    /// [`light_dark_toggle`](Self::light_dark_toggle) rather than assigning here, or it falls out
    /// of step with the mode.
    pub colors: ThemeColors,
    /// The colours of the light mode.
    #[educe(Default = DEFAULT_LIGHT_THEME_COLORS)]
    light_colors: ThemeColors,
    /// The colours of the dark mode.
    #[educe(Default = DEFAULT_DARK_THEME_COLORS)]
    dark_colors: ThemeColors,
    /// Which of the two sets is in use.
    mode: ThemeMode,

    /// The width of a scroll bar.
    #[educe(Default = Val::Px(8.0))]
    pub scroll_bar_width: Val,

    /// The font of body text: buttons, labels, links and list items.
    ///
    /// Defaults to Bevy's default font; the application loads its own and sets the handle.
    pub body_font: Handle<Font>,
    /// The font of titles, such as [`Label::Title`](crate::label::Label::Title).
    ///
    /// Defaults to Bevy's default font; the application loads its own and sets the handle.
    pub display_font: Handle<Font>,

    /// The size of a title, in the display font.
    #[educe(Default = 22.0)]
    pub font_size_title: f32,
    /// The size of the title of a row.
    #[educe(Default = 16.0)]
    pub font_size_row_title: f32,
    /// The size of body text.
    #[educe(Default = 15.0)]
    pub font_size_body: f32,
    /// The size of secondary text: buttons, labels and list items.
    #[educe(Default = 14.0)]
    pub font_size_secondary: f32,
    /// The size of the text of small controls.
    #[educe(Default = 12.0)]
    pub font_size_small: f32,
    /// The size of captions, overlines and badges.
    #[educe(Default = 10.0)]
    pub font_size_caption: f32,
    /// The size of the text of a filter chip.
    #[educe(Default = 13.0)]
    pub font_size_chip: f32,

    #[educe(Default = ShadowStyle {
        color: Color::BLACK,
        x_offset: Val::Px(2.0),
        y_offset: Val::Px(4.0),
        spread_radius: Val::Px(0.0),
        blur_radius:Val::Px(8.0),}
    ,)]
    shadow_style: ShadowStyle,

    /// The padding inside an item of a list.
    #[educe(Default = UiRect::axes(Val::Px(12.0), Val::Px(8.0)))]
    pub item_padding: UiRect,

    /// The height of a compact [`PushButton`](crate::push_button::PushButton).
    #[educe(Default = Val::Px(28.0))]
    pub compact_button_height: Val,
    /// The padding inside a compact [`PushButton`](crate::push_button::PushButton).
    #[educe(Default = UiRect::axes(Val::Px(10.0), Val::Px(0.0)))]
    pub compact_button_padding: UiRect,
    /// The radius of the corners of a compact [`PushButton`](crate::push_button::PushButton).
    #[educe(Default = Val::Px(6.0))]
    pub compact_button_radius: Val,

    /// The padding inside a colour push button.
    #[educe(Default = UiRect::axes(Val::Px(6.0), Val::Px(4.0)))]
    pub color_button_padding: UiRect,
}

impl Theme {
    /// The shadow of popups and cards.
    #[must_use]
    pub const fn large_shadow(&self) -> ShadowStyle {
        let mut s = self.shadow_style;
        s.color = self.colors.large_shadow;
        s
    }
    /// The shadow of buttons.
    #[must_use]
    pub const fn small_shadow(&self) -> ShadowStyle {
        let mut s = self.shadow_style;
        s.color = self.colors.small_shadow;
        s
    }

    /// A light-mode theme with the given pair of colour sets, everything else default.
    #[must_use]
    pub fn with_colors(light: ThemeColors, dark: ThemeColors) -> Self {
        Self {
            colors: light,
            light_colors: light,
            dark_colors: dark,
            ..Self::default()
        }
    }

    /// Which of the light and dark colour sets is in use.
    #[must_use]
    pub const fn mode(&self) -> ThemeMode {
        self.mode
    }

    /// Uses the light or the dark colour set.
    pub const fn set_mode(&mut self, mode: ThemeMode) {
        self.mode = mode;
        self.colors = match mode {
            ThemeMode::Light => self.light_colors,
            ThemeMode::Dark => self.dark_colors,
        };
    }

    /// Switches between the light and the dark mode, keeping the application's own colour pair.
    pub const fn light_dark_toggle(&mut self) {
        self.set_mode(match self.mode {
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::Light,
        });
    }
}

/// Whether a [`Theme`] uses its light or its dark colours.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Reflect)]
pub enum ThemeMode {
    /// The light colours.
    #[default]
    Light,
    /// The dark colours.
    Dark,
}

/// The colour slots of a [`Theme`].
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
pub struct ThemeColors {
    /// The background of widgets and the window.
    pub background_primary: Color,
    /// A secondary background, such as a hovered button or a scroll bar track.
    pub background_secondary: Color,
    /// The dimming behind a popup that blocks input.
    pub background_blocker: Color,
    /// The background of a selected item.
    pub selected_item: Color,
    /// The background of a hovered item.
    pub hovered_item: Color,
    /// The background of a selected item that is hovered.
    pub selected_hovered_item: Color,
    /// The colour of borders.
    pub border: Color,
    /// The colour of the border of a selected or hovered widget.
    pub border_highlight: Color,
    /// The colour of text.
    pub text_primary: Color,
    /// The colour of the text of a disabled widget.
    pub text_disabled: Color,
    /// The colour of a link.
    pub link: Color,
    /// The colour of a link while it is pressed.
    pub link_selected: Color,
    /// The colour of the large shadow.
    pub large_shadow: Color,
    /// The colour of the small shadow.
    pub small_shadow: Color,
    /// The colour of the ring around the widget that has keyboard focus.
    pub focus_ring: Color,
    /// The colour of a scroll bar thumb.
    pub scroll_thumb_background: Color,
    /// Secondary text, such as descriptions and counts.
    pub text_secondary: Color,
    /// Tertiary text, such as captions.
    pub text_tertiary: Color,
    /// The colour of divider lines.
    pub divider: Color,
    /// The fill of the active tool.
    pub accent: Color,
    /// The text on [`accent`](Self::accent).
    pub accent_text: Color,
    /// A subtle accent fill, such as a pressed button.
    pub accent_subtle: Color,
    /// The text on [`accent_subtle`](Self::accent_subtle).
    pub accent_subtle_text: Color,
    /// The bar on [`accent_subtle`](Self::accent_subtle).
    pub accent_subtle_bar: Color,
    /// The ring around a selected widget.
    pub selection_ring: Color,
    /// The strong hover of a selected row.
    pub hovered_selected_item: Color,
    /// The fill of a dangerous action.
    pub danger: Color,
    /// The text on [`danger`](Self::danger).
    pub danger_text: Color,
    /// The text of a dangerous action while hovered.
    pub danger_hover_text: Color,
    /// The background of a warning.
    pub warning_background: Color,
    /// The border of a warning.
    pub warning_border: Color,
    /// The text of a warning.
    pub warning_text: Color,
    /// The background of an informational status, such as running.
    pub info_background: Color,
    /// The text of an informational status.
    pub info_text: Color,
    /// The background of a waiting status.
    pub wait_background: Color,
    /// The text of a waiting status.
    pub wait_text: Color,
    /// The background of a neutral pill or badge.
    pub pill_background: Color,
    /// The text of a neutral pill or badge.
    pub pill_text: Color,
    /// The track of a segmented control.
    pub segmented_track: Color,
    /// The selected segment of a segmented control.
    pub segmented_selected: Color,
    /// The text of the selected segment.
    pub segmented_selected_text: Color,
    /// The surface of a card in a gallery.
    pub card_surface: Color,
    /// The border of a filter chip.
    pub chip_border: Color,
    /// The fill of the active filter chip.
    pub chip_on: Color,
    /// The text on [`chip_on`](Self::chip_on).
    pub chip_on_ink: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        DEFAULT_LIGHT_THEME_COLORS
    }
}

/// The default colours of the light theme.
pub const DEFAULT_LIGHT_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::Srgba(GRAY_50),
    background_secondary: Color::Srgba(GRAY_200),
    background_blocker: Color::Srgba(Srgba::new(
        GRAY_500.red,
        GRAY_500.green,
        GRAY_500.blue,
        0.75,
    )),
    selected_item: Color::Srgba(BLUE_500),
    hovered_item: Color::Srgba(BLUE_200),
    selected_hovered_item: Color::Srgba(BLUE_600),
    border: Color::Srgba(GRAY_200),
    border_highlight: Color::Srgba(BLUE_400),
    text_primary: Color::Srgba(GRAY_900),
    text_disabled: Color::Srgba(GRAY_400),
    link: Color::Srgba(BLUE_600),
    link_selected: Color::Srgba(BLUE_800),
    large_shadow: SHADOW_LG_LIGHT_MODE,
    small_shadow: SHADOW_SM,
    focus_ring: Color::Srgba(BLUE_600),
    scroll_thumb_background: Color::Srgba(GRAY_400),
    text_secondary: Color::srgb_u8(0x5C, 0x62, 0x6A),
    text_tertiary: Color::srgb_u8(0x6E, 0x74, 0x7C),
    divider: Color::srgb_u8(0xE5, 0xE1, 0xD7),
    accent: Color::srgb_u8(0xF8, 0xD7, 0xA4),
    accent_text: Color::srgb_u8(0x3D, 0x2A, 0x08),
    accent_subtle: Color::srgb_u8(0xF8, 0xE6, 0xC8),
    accent_subtle_text: Color::srgb_u8(0x7A, 0x4E, 0x0E),
    accent_subtle_bar: Color::srgb_u8(0xE9, 0xB6, 0x6A),
    selection_ring: Color::srgb_u8(0xE9, 0xB6, 0x6A),
    hovered_selected_item: Color::srgb_u8(0xE7, 0xE3, 0xD9),
    danger: Color::srgb_u8(0xF6, 0xC3, 0xB6),
    danger_text: Color::srgb_u8(0x5C, 0x1A, 0x0C),
    danger_hover_text: Color::srgb_u8(0xB8, 0x43, 0x2B),
    warning_background: Color::srgb_u8(0xFB, 0xE1, 0xDC),
    warning_border: Color::srgb_u8(0xE3, 0x9A, 0x8F),
    warning_text: Color::srgb_u8(0xB3, 0x26, 0x1E),
    info_background: Color::srgb_u8(0xD8, 0xE8, 0xF7),
    info_text: Color::srgb_u8(0x1F, 0x4E, 0x7A),
    wait_background: Color::srgb_u8(0xF9, 0xE5, 0xC2),
    wait_text: Color::srgb_u8(0x6E, 0x46, 0x08),
    pill_background: Color::srgb_u8(0xEC, 0xE8, 0xDF),
    pill_text: Color::srgb_u8(0x43, 0x48, 0x4F),
    segmented_track: Color::srgb_u8(0xEC, 0xE8, 0xDF),
    segmented_selected: Color::srgb_u8(0xFF, 0xFF, 0xFF),
    segmented_selected_text: Color::srgb_u8(0x22, 0x25, 0x2A),
    card_surface: Color::srgb_u8(0xFF, 0xFF, 0xFF),
    chip_border: Color::srgb_u8(0xD3, 0xCE, 0xC2),
    chip_on: Color::srgb_u8(0xD5, 0xE5, 0xF5),
    chip_on_ink: Color::srgb_u8(0x1E, 0x3F, 0x62),
};
/// The default colours of the dark theme.
pub const DEFAULT_DARK_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::Srgba(GRAY_950),
    background_secondary: Color::Srgba(GRAY_800),
    background_blocker: Color::Srgba(Srgba::new(
        GRAY_950.red,
        GRAY_950.green,
        GRAY_950.blue,
        0.75,
    )),
    selected_item: Color::Srgba(BLUE_800),
    hovered_item: Color::Srgba(BLUE_900),
    selected_hovered_item: Color::Srgba(BLUE_700),
    border: Color::Srgba(GRAY_700),
    border_highlight: Color::Srgba(BLUE_800),
    text_primary: Color::Srgba(GRAY_100),
    text_disabled: Color::Srgba(GRAY_600),
    link: Color::Srgba(BLUE_400),
    link_selected: Color::Srgba(BLUE_200),
    large_shadow: SHADOW_LG_DARK_MODE,
    small_shadow: SHADOW_SM_DARK_MODE,
    focus_ring: Color::Srgba(BLUE_400),
    scroll_thumb_background: Color::Srgba(GRAY_600),
    text_secondary: Color::srgb_u8(0xA3, 0xA9, 0xB1),
    text_tertiary: Color::srgb_u8(0x8A, 0x91, 0x9A),
    divider: Color::srgb_u8(0x2A, 0x2E, 0x35),
    accent: Color::srgb_u8(0xF2, 0xA9, 0x3B),
    accent_text: Color::srgb_u8(0x17, 0x13, 0x0A),
    accent_subtle: Color::srgb_u8(0x2C, 0x31, 0x38),
    accent_subtle_text: Color::srgb_u8(0xF2, 0xA9, 0x3B),
    accent_subtle_bar: Color::srgb_u8(0xF2, 0xA9, 0x3B),
    selection_ring: Color::srgb_u8(0xF2, 0xA9, 0x3B),
    hovered_selected_item: Color::srgb_u8(0x2C, 0x31, 0x38),
    danger: Color::srgb_u8(0xE3, 0x66, 0x4A),
    danger_text: Color::srgb_u8(0x1C, 0x0E, 0x0A),
    danger_hover_text: Color::srgb_u8(0xF3, 0x9A, 0x85),
    warning_background: Color::srgb_u8(0x3A, 0x1D, 0x1B),
    warning_border: Color::srgb_u8(0xE2, 0x57, 0x4C),
    warning_text: Color::srgb_u8(0xFF, 0x8A, 0x80),
    info_background: Color::srgb_u8(0x1C, 0x31, 0x44),
    info_text: Color::srgb_u8(0x8E, 0xC6, 0xF2),
    wait_background: Color::srgb_u8(0x3A, 0x2D, 0x17),
    wait_text: Color::srgb_u8(0xF6, 0xC1, 0x71),
    pill_background: Color::srgb_u8(0x2A, 0x2E, 0x35),
    pill_text: Color::srgb_u8(0xC9, 0xCE, 0xD6),
    segmented_track: Color::srgb_u8(0x22, 0x26, 0x2C),
    segmented_selected: Color::srgb_u8(0xE8, 0xE6, 0xE1),
    segmented_selected_text: Color::srgb_u8(0x14, 0x17, 0x1C),
    card_surface: Color::srgb_u8(0x22, 0x26, 0x2C),
    chip_border: Color::srgb_u8(0x3A, 0x40, 0x48),
    chip_on: Color::srgb_u8(0xE8, 0xE6, 0xE1),
    chip_on_ink: Color::srgb_u8(0x14, 0x17, 0x1C),
};

/// A small shadow.
pub const SHADOW_SM: Color = Color::srgba(0.0, 0.0, 0.0, 0.05);
/// A medium shadow.
pub const SHADOW_MD: Color = Color::srgba(0.0, 0.0, 0.0, 0.1);
/// A large shadow.
pub const SHADOW_LG: Color = Color::srgba(0.0, 0.0, 0.0, 0.15);
/// An extra large shadow.
pub const SHADOW_XL: Color = Color::srgba(0.0, 0.0, 0.0, 0.25);

/// A small shadow that shows up on a dark background.
pub const SHADOW_SM_DARK_MODE: Color = Color::srgba(1.0, 1.0, 1.0, 0.03);
/// A large shadow on a dark background: black, and heavy enough to show.
pub const SHADOW_LG_DARK_MODE: Color = Color::srgba(0.0, 0.0, 0.0, 0.35);
/// The large shadow on a light background: a warm dark.
pub const SHADOW_LG_LIGHT_MODE: Color =
    Color::srgba(60.0 / 255.0, 50.0 / 255.0, 30.0 / 255.0, 0.14);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_default_themes_differ_in_every_new_slot() {
        let (l, d) = (DEFAULT_LIGHT_THEME_COLORS, DEFAULT_DARK_THEME_COLORS);
        let pairs = [
            ("text_secondary", l.text_secondary, d.text_secondary),
            ("text_tertiary", l.text_tertiary, d.text_tertiary),
            ("divider", l.divider, d.divider),
            ("accent", l.accent, d.accent),
            ("accent_text", l.accent_text, d.accent_text),
            ("accent_subtle", l.accent_subtle, d.accent_subtle),
            (
                "accent_subtle_text",
                l.accent_subtle_text,
                d.accent_subtle_text,
            ),
            (
                "accent_subtle_bar",
                l.accent_subtle_bar,
                d.accent_subtle_bar,
            ),
            ("selection_ring", l.selection_ring, d.selection_ring),
            (
                "hovered_selected_item",
                l.hovered_selected_item,
                d.hovered_selected_item,
            ),
            ("danger", l.danger, d.danger),
            ("danger_text", l.danger_text, d.danger_text),
            (
                "danger_hover_text",
                l.danger_hover_text,
                d.danger_hover_text,
            ),
            (
                "warning_background",
                l.warning_background,
                d.warning_background,
            ),
            ("warning_border", l.warning_border, d.warning_border),
            ("warning_text", l.warning_text, d.warning_text),
            ("info_background", l.info_background, d.info_background),
            ("info_text", l.info_text, d.info_text),
            ("wait_background", l.wait_background, d.wait_background),
            ("wait_text", l.wait_text, d.wait_text),
            ("pill_background", l.pill_background, d.pill_background),
            ("pill_text", l.pill_text, d.pill_text),
            ("segmented_track", l.segmented_track, d.segmented_track),
            (
                "segmented_selected",
                l.segmented_selected,
                d.segmented_selected,
            ),
            (
                "segmented_selected_text",
                l.segmented_selected_text,
                d.segmented_selected_text,
            ),
            ("card_surface", l.card_surface, d.card_surface),
            ("chip_border", l.chip_border, d.chip_border),
            ("chip_on", l.chip_on, d.chip_on),
            ("chip_on_ink", l.chip_on_ink, d.chip_on_ink),
        ];
        for (name, light, dark) in pairs {
            assert_ne!(light, dark, "{name} is the same in both themes");
        }
    }

    fn custom_pair() -> (ThemeColors, ThemeColors) {
        let light = ThemeColors {
            accent: Color::srgb(1.0, 0.0, 0.0),
            ..DEFAULT_LIGHT_THEME_COLORS
        };
        let dark = ThemeColors {
            accent: Color::srgb(0.0, 0.0, 1.0),
            ..DEFAULT_DARK_THEME_COLORS
        };
        (light, dark)
    }

    #[test]
    fn a_default_theme_is_the_light_defaults() {
        let theme = Theme::default();
        assert_eq!(theme.mode(), ThemeMode::Light);
        assert_eq!(theme.colors, DEFAULT_LIGHT_THEME_COLORS);
    }

    #[test]
    fn set_mode_switches_between_a_custom_pair() {
        let (light, dark) = custom_pair();
        let mut theme = Theme::with_colors(light, dark);
        assert_eq!(theme.colors, light);
        theme.set_mode(ThemeMode::Dark);
        assert_eq!((theme.mode(), theme.colors), (ThemeMode::Dark, dark));
        theme.set_mode(ThemeMode::Dark);
        assert_eq!(theme.colors, dark);
        theme.set_mode(ThemeMode::Light);
        assert_eq!((theme.mode(), theme.colors), (ThemeMode::Light, light));
    }

    #[test]
    fn light_dark_toggle_switches_between_a_custom_pair() {
        let (light, dark) = custom_pair();
        let mut theme = Theme::with_colors(light, dark);
        theme.light_dark_toggle();
        assert_eq!(theme.colors, dark);
        theme.light_dark_toggle();
        assert_eq!(theme.colors, light);
    }

    #[test]
    fn the_large_shadow_is_dark_in_both_themes() {
        for colors in [DEFAULT_LIGHT_THEME_COLORS, DEFAULT_DARK_THEME_COLORS] {
            let shadow = colors.large_shadow.to_srgba();
            assert!(shadow.red < 0.3 && shadow.green < 0.3 && shadow.blue < 0.3);
        }
    }
}
