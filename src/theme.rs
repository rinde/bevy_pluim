use bevy::color::Color;
use bevy::color::palettes::tailwind::BLUE_200;
use bevy::color::palettes::tailwind::BLUE_500;
use bevy::color::palettes::tailwind::BLUE_600;
use bevy::color::palettes::tailwind::GRAY_200;
use bevy::color::palettes::tailwind::GRAY_900;
use bevy::ecs::change_detection::DetectChanges;
use bevy::ecs::resource::Resource;
use bevy::ecs::system::Res;
use bevy::ui::Val;
use educe::Educe;

#[derive(Resource, Clone, Copy, Debug, Educe)]
#[educe(Default)]
pub struct Theme {
    #[educe(Default = Val::Px(10.0))]
    pub corner_radius: Val,

    pub colors: ThemeColors,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ThemeColors {
    pub background_primary: Color,
    pub selected_item: Color,
    pub hovered_item: Color,
    pub selected_hovered_item: Color,
    pub border: Color,
    pub text_primary: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        DEFAULT_LIGHT_THEME_COLORS
    }
}

const DEFAULT_LIGHT_THEME_COLORS: ThemeColors = ThemeColors {
    background_primary: Color::WHITE,
    selected_item: Color::Srgba(BLUE_500),
    hovered_item: Color::Srgba(BLUE_200),
    selected_hovered_item: Color::Srgba(BLUE_600),
    border: Color::Srgba(GRAY_200),
    text_primary: Color::Srgba(GRAY_900),
};

pub(crate) fn on_theme_change(theme: Res<Theme>) {
    if theme.is_changed() {
        println!("theme changed");
        // TODO dispatch event
    }
}
