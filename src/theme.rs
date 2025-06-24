use bevy::color::Color;
use bevy::color::palettes::tailwind::BLUE_200;
use bevy::color::palettes::tailwind::BLUE_500;
use bevy::color::palettes::tailwind::BLUE_600;
use bevy::color::palettes::tailwind::GRAY_50;
use bevy::color::palettes::tailwind::GRAY_200;
use bevy::color::palettes::tailwind::GRAY_900;
use bevy::ecs::change_detection::DetectChanges;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::Res;
use bevy::ui::ShadowStyle;
use bevy::ui::Val;
use educe::Educe;

#[derive(Resource, Clone, Copy, Debug, Educe)]
#[educe(Default)]
pub struct Theme {
    #[educe(Default = Val::Px(10.0))]
    pub corner_radius: Val,
    #[educe(Default = Val::Px(1.0))]
    pub border_thickness: Val,

    pub colors: ThemeColors,

    #[educe(Default = ShadowStyle {
        color: Color::BLACK,
        x_offset: Val::Px(2.0),
        y_offset: Val::Px(4.0),
        spread_radius: Val::Px(0.0),
        blur_radius:Val::Px(8.0),}
    ,)]
    shadow_style: ShadowStyle,
}

impl Theme {
    #[must_use]
    pub const fn shadow_style(&self) -> ShadowStyle {
        let mut s = self.shadow_style;
        s.color = self.colors.shadow;
        s
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
    pub text_primary: Color,
    pub shadow: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        DEFAULT_LIGHT_THEME_COLORS
    }
}

const DEFAULT_LIGHT_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::WHITE,
    background_secondary: Color::Srgba(GRAY_50),
    selected_item: Color::Srgba(BLUE_500),
    hovered_item: Color::Srgba(BLUE_200),
    selected_hovered_item: Color::Srgba(BLUE_600),
    border: Color::Srgba(GRAY_200),
    text_primary: Color::Srgba(GRAY_900),
    shadow: SHADOW_LG,
};

pub(crate) fn on_theme_change(theme: Res<Theme>) {
    if theme.is_changed() {
        println!("theme changed");
        // TODO dispatch event
    }
}

pub const SHADOW_SM: Color = Color::srgba(0.0, 0.0, 0.0, 0.05);
pub const SHADOW_MD: Color = Color::srgba(0.0, 0.0, 0.0, 0.1);
pub const SHADOW_LG: Color = Color::srgba(0.0, 0.0, 0.0, 0.15);
pub const SHADOW_XL: Color = Color::srgba(0.0, 0.0, 0.0, 0.25);
