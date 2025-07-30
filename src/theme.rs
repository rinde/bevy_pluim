use bevy::color::palettes::tailwind::{
    BLUE_200, BLUE_400, BLUE_500, BLUE_600, BLUE_700, BLUE_800, BLUE_900, GRAY_50, GRAY_100,
    GRAY_200, GRAY_400, GRAY_600, GRAY_800, GRAY_900, GRAY_950,
};
use bevy::prelude::*;
use educe::Educe;

#[derive(Resource, Clone, Copy, Debug, Educe)]
#[educe(Default)]
pub struct Theme {
    #[educe(Default = Val::Px(10.0))]
    pub corner_radius: Val,
    #[educe(Default = Val::Px(1.0))]
    pub border_thickness: Val,

    pub colors: ThemeColors,

    #[educe(Default = Val::Px(8.0))]
    pub scroll_bar_width: Val,

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

    #[educe(Default = UiRect::axes(Val::Px(12.0), Val::Px(8.0)))]
    pub item_padding: UiRect,

    #[educe(Default = UiRect::axes(Val::Px(6.0), Val::Px(4.0)))]
    pub color_button_padding: UiRect,
}

impl Theme {
    #[must_use]
    pub const fn large_shadow(&self) -> ShadowStyle {
        let mut s = self.shadow_style;
        s.color = self.colors.large_shadow;
        s
    }
    #[must_use]
    pub const fn small_shadow(&self) -> ShadowStyle {
        let mut s = self.shadow_style;
        s.color = self.colors.small_shadow;
        s
    }

    pub fn light_dark_toggle(&mut self) {
        self.colors = if self.colors == DEFAULT_LIGHT_THEME_COLORS {
            DEFAULT_DARK_THEME_COLORS
        } else {
            DEFAULT_LIGHT_THEME_COLORS
        };
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeColors {
    pub background_primary: Color,
    pub background_secondary: Color,
    pub selected_item: Color,
    pub hovered_item: Color,
    pub selected_hovered_item: Color,
    pub border: Color,
    pub border_highlight: Color,
    pub text_primary: Color,
    pub large_shadow: Color,
    pub small_shadow: Color,
    pub scroll_thumb_background: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        DEFAULT_LIGHT_THEME_COLORS
    }
}

pub const DEFAULT_LIGHT_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::Srgba(GRAY_50),
    background_secondary: Color::Srgba(GRAY_200),
    selected_item: Color::Srgba(BLUE_500),
    hovered_item: Color::Srgba(BLUE_200),
    selected_hovered_item: Color::Srgba(BLUE_600),
    border: Color::Srgba(GRAY_200),
    border_highlight: Color::Srgba(BLUE_400),
    text_primary: Color::Srgba(GRAY_900),
    large_shadow: SHADOW_LG,
    small_shadow: SHADOW_SM,
    scroll_thumb_background: Color::Srgba(GRAY_400),
};
pub const DEFAULT_DARK_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::Srgba(GRAY_950),
    background_secondary: Color::Srgba(GRAY_800),
    selected_item: Color::Srgba(BLUE_800),
    hovered_item: Color::Srgba(BLUE_900),
    selected_hovered_item: Color::Srgba(BLUE_700),
    border: Color::Srgba(GRAY_800),
    border_highlight: Color::Srgba(BLUE_800),
    text_primary: Color::Srgba(GRAY_100),
    large_shadow: SHADOW_LG_DARK_MODE,
    small_shadow: SHADOW_SM_DARK_MODE,
    scroll_thumb_background: Color::Srgba(GRAY_600),
};

pub const SHADOW_SM: Color = Color::srgba(0.0, 0.0, 0.0, 0.05);
pub const SHADOW_MD: Color = Color::srgba(0.0, 0.0, 0.0, 0.1);
pub const SHADOW_LG: Color = Color::srgba(0.0, 0.0, 0.0, 0.15);
pub const SHADOW_XL: Color = Color::srgba(0.0, 0.0, 0.0, 0.25);

pub const SHADOW_SM_DARK_MODE: Color = Color::srgba(1.0, 1.0, 1.0, 0.03);
pub const SHADOW_LG_DARK_MODE: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);
