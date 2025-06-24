use bevy::color::Color;
use bevy::color::palettes::tailwind::BLUE_200;
use bevy::color::palettes::tailwind::BLUE_500;
use bevy::color::palettes::tailwind::BLUE_600;
use bevy::color::palettes::tailwind::BLUE_700;
use bevy::color::palettes::tailwind::BLUE_800;
use bevy::color::palettes::tailwind::BLUE_900;
use bevy::color::palettes::tailwind::GRAY_100;
use bevy::color::palettes::tailwind::GRAY_200;
use bevy::color::palettes::tailwind::GRAY_800;
use bevy::color::palettes::tailwind::GRAY_900;
use bevy::color::palettes::tailwind::GRAY_950;
use bevy::ecs::change_detection::DetectChanges;
use bevy::ecs::event::Event;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::Commands;
use bevy::ecs::system::Res;
use bevy::ui::ShadowStyle;
use bevy::ui::UiRect;
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

#[derive(Event, Debug, Copy, Clone)]
pub struct ThemeChangedEvent;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeColors {
    pub background_primary: Color,
    pub background_secondary: Color,
    pub selected_item: Color,
    pub hovered_item: Color,
    pub selected_hovered_item: Color,
    pub border: Color,
    pub text_primary: Color,
    pub large_shadow: Color,
    pub small_shadow: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        DEFAULT_LIGHT_THEME_COLORS
    }
}

pub const DEFAULT_LIGHT_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::WHITE,
    background_secondary: Color::Srgba(GRAY_200),
    selected_item: Color::Srgba(BLUE_500),
    hovered_item: Color::Srgba(BLUE_200),
    selected_hovered_item: Color::Srgba(BLUE_600),
    border: Color::Srgba(GRAY_200),
    text_primary: Color::Srgba(GRAY_900),
    large_shadow: SHADOW_LG,
    small_shadow: SHADOW_SM,
};
pub const DEFAULT_DARK_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::BLACK,
    background_secondary: Color::Srgba(GRAY_950),
    selected_item: Color::Srgba(BLUE_800),
    hovered_item: Color::Srgba(BLUE_900),
    selected_hovered_item: Color::Srgba(BLUE_700),
    border: Color::Srgba(GRAY_800),
    text_primary: Color::Srgba(GRAY_100),
    large_shadow: SHADOW_LG,
    small_shadow: SHADOW_SM,
};

pub(crate) fn on_theme_change(theme: Res<Theme>, mut commands: Commands) {
    if theme.is_changed() {
        commands.trigger(ThemeChangedEvent);
    }
}

pub const SHADOW_SM: Color = Color::srgba(0.0, 0.0, 0.0, 0.05);
pub const SHADOW_MD: Color = Color::srgba(0.0, 0.0, 0.0, 0.1);
pub const SHADOW_LG: Color = Color::srgba(0.0, 0.0, 0.0, 0.15);
pub const SHADOW_XL: Color = Color::srgba(0.0, 0.0, 0.0, 0.25);
