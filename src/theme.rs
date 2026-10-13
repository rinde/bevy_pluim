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
#[derive(Resource, Clone, Copy, Debug, Educe, Reflect)]
#[reflect(Resource)]
#[educe(Default)]
pub struct Theme {
    /// The radius of rounded corners.
    #[educe(Default = Val::Px(10.0))]
    pub corner_radius: Val,
    /// The thickness of borders.
    #[educe(Default = Val::Px(1.0))]
    pub border_thickness: Val,

    /// The colours.
    pub colors: ThemeColors,

    /// The width of a scroll bar.
    #[educe(Default = Val::Px(8.0))]
    pub scroll_bar_width: Val,

    /// The size of regular text.
    #[educe(Default = 14.0)]
    pub font_size: f32,

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

    /// Switches between [`DEFAULT_LIGHT_THEME_COLORS`] and [`DEFAULT_DARK_THEME_COLORS`].
    pub fn light_dark_toggle(&mut self) {
        self.colors = if self.colors == DEFAULT_LIGHT_THEME_COLORS {
            DEFAULT_DARK_THEME_COLORS
        } else {
            DEFAULT_LIGHT_THEME_COLORS
        };
    }
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
    large_shadow: SHADOW_LG,
    small_shadow: SHADOW_SM,
    focus_ring: Color::Srgba(BLUE_600),
    scroll_thumb_background: Color::Srgba(GRAY_400),
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
/// A large shadow that shows up on a dark background.
pub const SHADOW_LG_DARK_MODE: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);
