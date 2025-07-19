use bevy::color::Color;
use bevy::color::palettes::tailwind::AMBER_50;
use bevy::color::palettes::tailwind::AMBER_100;
use bevy::color::palettes::tailwind::AMBER_200;
use bevy::color::palettes::tailwind::AMBER_300;
use bevy::color::palettes::tailwind::AMBER_400;
use bevy::color::palettes::tailwind::AMBER_500;
use bevy::color::palettes::tailwind::AMBER_600;
use bevy::color::palettes::tailwind::AMBER_700;
use bevy::color::palettes::tailwind::AMBER_800;
use bevy::color::palettes::tailwind::AMBER_900;
use bevy::color::palettes::tailwind::AMBER_950;
use bevy::color::palettes::tailwind::BLUE_50;
use bevy::color::palettes::tailwind::BLUE_100;
use bevy::color::palettes::tailwind::BLUE_200;
use bevy::color::palettes::tailwind::BLUE_300;
use bevy::color::palettes::tailwind::BLUE_400;
use bevy::color::palettes::tailwind::BLUE_500;
use bevy::color::palettes::tailwind::BLUE_600;
use bevy::color::palettes::tailwind::BLUE_700;
use bevy::color::palettes::tailwind::BLUE_800;
use bevy::color::palettes::tailwind::BLUE_900;
use bevy::color::palettes::tailwind::BLUE_950;
use bevy::color::palettes::tailwind::CYAN_50;
use bevy::color::palettes::tailwind::CYAN_100;
use bevy::color::palettes::tailwind::CYAN_200;
use bevy::color::palettes::tailwind::CYAN_300;
use bevy::color::palettes::tailwind::CYAN_400;
use bevy::color::palettes::tailwind::CYAN_500;
use bevy::color::palettes::tailwind::CYAN_600;
use bevy::color::palettes::tailwind::CYAN_700;
use bevy::color::palettes::tailwind::CYAN_800;
use bevy::color::palettes::tailwind::CYAN_900;
use bevy::color::palettes::tailwind::CYAN_950;
use bevy::color::palettes::tailwind::EMERALD_50;
use bevy::color::palettes::tailwind::EMERALD_100;
use bevy::color::palettes::tailwind::EMERALD_200;
use bevy::color::palettes::tailwind::EMERALD_300;
use bevy::color::palettes::tailwind::EMERALD_400;
use bevy::color::palettes::tailwind::EMERALD_500;
use bevy::color::palettes::tailwind::EMERALD_600;
use bevy::color::palettes::tailwind::EMERALD_700;
use bevy::color::palettes::tailwind::EMERALD_800;
use bevy::color::palettes::tailwind::EMERALD_900;
use bevy::color::palettes::tailwind::EMERALD_950;
use bevy::color::palettes::tailwind::FUCHSIA_50;
use bevy::color::palettes::tailwind::FUCHSIA_100;
use bevy::color::palettes::tailwind::FUCHSIA_200;
use bevy::color::palettes::tailwind::FUCHSIA_300;
use bevy::color::palettes::tailwind::FUCHSIA_400;
use bevy::color::palettes::tailwind::FUCHSIA_500;
use bevy::color::palettes::tailwind::FUCHSIA_600;
use bevy::color::palettes::tailwind::FUCHSIA_700;
use bevy::color::palettes::tailwind::FUCHSIA_800;
use bevy::color::palettes::tailwind::FUCHSIA_900;
use bevy::color::palettes::tailwind::FUCHSIA_950;
use bevy::color::palettes::tailwind::GRAY_50;
use bevy::color::palettes::tailwind::GRAY_100;
use bevy::color::palettes::tailwind::GRAY_200;
use bevy::color::palettes::tailwind::GRAY_300;
use bevy::color::palettes::tailwind::GRAY_400;
use bevy::color::palettes::tailwind::GRAY_500;
use bevy::color::palettes::tailwind::GRAY_600;
use bevy::color::palettes::tailwind::GRAY_700;
use bevy::color::palettes::tailwind::GRAY_800;
use bevy::color::palettes::tailwind::GRAY_900;
use bevy::color::palettes::tailwind::GRAY_950;
use bevy::color::palettes::tailwind::GREEN_50;
use bevy::color::palettes::tailwind::GREEN_100;
use bevy::color::palettes::tailwind::GREEN_200;
use bevy::color::palettes::tailwind::GREEN_300;
use bevy::color::palettes::tailwind::GREEN_400;
use bevy::color::palettes::tailwind::GREEN_500;
use bevy::color::palettes::tailwind::GREEN_600;
use bevy::color::palettes::tailwind::GREEN_700;
use bevy::color::palettes::tailwind::GREEN_800;
use bevy::color::palettes::tailwind::GREEN_900;
use bevy::color::palettes::tailwind::GREEN_950;
use bevy::color::palettes::tailwind::INDIGO_50;
use bevy::color::palettes::tailwind::INDIGO_100;
use bevy::color::palettes::tailwind::INDIGO_200;
use bevy::color::palettes::tailwind::INDIGO_300;
use bevy::color::palettes::tailwind::INDIGO_400;
use bevy::color::palettes::tailwind::INDIGO_500;
use bevy::color::palettes::tailwind::INDIGO_600;
use bevy::color::palettes::tailwind::INDIGO_700;
use bevy::color::palettes::tailwind::INDIGO_800;
use bevy::color::palettes::tailwind::INDIGO_900;
use bevy::color::palettes::tailwind::INDIGO_950;
use bevy::color::palettes::tailwind::LIME_50;
use bevy::color::palettes::tailwind::LIME_100;
use bevy::color::palettes::tailwind::LIME_200;
use bevy::color::palettes::tailwind::LIME_300;
use bevy::color::palettes::tailwind::LIME_400;
use bevy::color::palettes::tailwind::LIME_500;
use bevy::color::palettes::tailwind::LIME_600;
use bevy::color::palettes::tailwind::LIME_700;
use bevy::color::palettes::tailwind::LIME_800;
use bevy::color::palettes::tailwind::LIME_900;
use bevy::color::palettes::tailwind::LIME_950;
use bevy::color::palettes::tailwind::NEUTRAL_50;
use bevy::color::palettes::tailwind::NEUTRAL_100;
use bevy::color::palettes::tailwind::NEUTRAL_200;
use bevy::color::palettes::tailwind::NEUTRAL_300;
use bevy::color::palettes::tailwind::NEUTRAL_400;
use bevy::color::palettes::tailwind::NEUTRAL_500;
use bevy::color::palettes::tailwind::NEUTRAL_600;
use bevy::color::palettes::tailwind::NEUTRAL_700;
use bevy::color::palettes::tailwind::NEUTRAL_800;
use bevy::color::palettes::tailwind::NEUTRAL_900;
use bevy::color::palettes::tailwind::NEUTRAL_950;
use bevy::color::palettes::tailwind::ORANGE_50;
use bevy::color::palettes::tailwind::ORANGE_100;
use bevy::color::palettes::tailwind::ORANGE_200;
use bevy::color::palettes::tailwind::ORANGE_300;
use bevy::color::palettes::tailwind::ORANGE_400;
use bevy::color::palettes::tailwind::ORANGE_500;
use bevy::color::palettes::tailwind::ORANGE_600;
use bevy::color::palettes::tailwind::ORANGE_700;
use bevy::color::palettes::tailwind::ORANGE_800;
use bevy::color::palettes::tailwind::ORANGE_900;
use bevy::color::palettes::tailwind::ORANGE_950;
use bevy::color::palettes::tailwind::PINK_50;
use bevy::color::palettes::tailwind::PINK_100;
use bevy::color::palettes::tailwind::PINK_200;
use bevy::color::palettes::tailwind::PINK_300;
use bevy::color::palettes::tailwind::PINK_400;
use bevy::color::palettes::tailwind::PINK_500;
use bevy::color::palettes::tailwind::PINK_600;
use bevy::color::palettes::tailwind::PINK_700;
use bevy::color::palettes::tailwind::PINK_800;
use bevy::color::palettes::tailwind::PINK_900;
use bevy::color::palettes::tailwind::PINK_950;
use bevy::color::palettes::tailwind::PURPLE_50;
use bevy::color::palettes::tailwind::PURPLE_100;
use bevy::color::palettes::tailwind::PURPLE_200;
use bevy::color::palettes::tailwind::PURPLE_300;
use bevy::color::palettes::tailwind::PURPLE_400;
use bevy::color::palettes::tailwind::PURPLE_500;
use bevy::color::palettes::tailwind::PURPLE_600;
use bevy::color::palettes::tailwind::PURPLE_700;
use bevy::color::palettes::tailwind::PURPLE_800;
use bevy::color::palettes::tailwind::PURPLE_900;
use bevy::color::palettes::tailwind::PURPLE_950;
use bevy::color::palettes::tailwind::RED_50;
use bevy::color::palettes::tailwind::RED_100;
use bevy::color::palettes::tailwind::RED_200;
use bevy::color::palettes::tailwind::RED_300;
use bevy::color::palettes::tailwind::RED_400;
use bevy::color::palettes::tailwind::RED_500;
use bevy::color::palettes::tailwind::RED_600;
use bevy::color::palettes::tailwind::RED_700;
use bevy::color::palettes::tailwind::RED_800;
use bevy::color::palettes::tailwind::RED_900;
use bevy::color::palettes::tailwind::RED_950;
use bevy::color::palettes::tailwind::ROSE_50;
use bevy::color::palettes::tailwind::ROSE_100;
use bevy::color::palettes::tailwind::ROSE_200;
use bevy::color::palettes::tailwind::ROSE_300;
use bevy::color::palettes::tailwind::ROSE_400;
use bevy::color::palettes::tailwind::ROSE_500;
use bevy::color::palettes::tailwind::ROSE_600;
use bevy::color::palettes::tailwind::ROSE_700;
use bevy::color::palettes::tailwind::ROSE_800;
use bevy::color::palettes::tailwind::ROSE_900;
use bevy::color::palettes::tailwind::ROSE_950;
use bevy::color::palettes::tailwind::SKY_50;
use bevy::color::palettes::tailwind::SKY_100;
use bevy::color::palettes::tailwind::SKY_200;
use bevy::color::palettes::tailwind::SKY_300;
use bevy::color::palettes::tailwind::SKY_400;
use bevy::color::palettes::tailwind::SKY_500;
use bevy::color::palettes::tailwind::SKY_600;
use bevy::color::palettes::tailwind::SKY_700;
use bevy::color::palettes::tailwind::SKY_800;
use bevy::color::palettes::tailwind::SKY_900;
use bevy::color::palettes::tailwind::SKY_950;
use bevy::color::palettes::tailwind::SLATE_50;
use bevy::color::palettes::tailwind::SLATE_100;
use bevy::color::palettes::tailwind::SLATE_200;
use bevy::color::palettes::tailwind::SLATE_300;
use bevy::color::palettes::tailwind::SLATE_400;
use bevy::color::palettes::tailwind::SLATE_500;
use bevy::color::palettes::tailwind::SLATE_600;
use bevy::color::palettes::tailwind::SLATE_700;
use bevy::color::palettes::tailwind::SLATE_800;
use bevy::color::palettes::tailwind::SLATE_900;
use bevy::color::palettes::tailwind::SLATE_950;
use bevy::color::palettes::tailwind::STONE_50;
use bevy::color::palettes::tailwind::STONE_100;
use bevy::color::palettes::tailwind::STONE_200;
use bevy::color::palettes::tailwind::STONE_300;
use bevy::color::palettes::tailwind::STONE_400;
use bevy::color::palettes::tailwind::STONE_500;
use bevy::color::palettes::tailwind::STONE_600;
use bevy::color::palettes::tailwind::STONE_700;
use bevy::color::palettes::tailwind::STONE_800;
use bevy::color::palettes::tailwind::STONE_900;
use bevy::color::palettes::tailwind::STONE_950;
use bevy::color::palettes::tailwind::TEAL_50;
use bevy::color::palettes::tailwind::TEAL_100;
use bevy::color::palettes::tailwind::TEAL_200;
use bevy::color::palettes::tailwind::TEAL_300;
use bevy::color::palettes::tailwind::TEAL_400;
use bevy::color::palettes::tailwind::TEAL_500;
use bevy::color::palettes::tailwind::TEAL_600;
use bevy::color::palettes::tailwind::TEAL_700;
use bevy::color::palettes::tailwind::TEAL_800;
use bevy::color::palettes::tailwind::TEAL_900;
use bevy::color::palettes::tailwind::TEAL_950;
use bevy::color::palettes::tailwind::VIOLET_50;
use bevy::color::palettes::tailwind::VIOLET_100;
use bevy::color::palettes::tailwind::VIOLET_200;
use bevy::color::palettes::tailwind::VIOLET_300;
use bevy::color::palettes::tailwind::VIOLET_400;
use bevy::color::palettes::tailwind::VIOLET_500;
use bevy::color::palettes::tailwind::VIOLET_600;
use bevy::color::palettes::tailwind::VIOLET_700;
use bevy::color::palettes::tailwind::VIOLET_800;
use bevy::color::palettes::tailwind::VIOLET_900;
use bevy::color::palettes::tailwind::VIOLET_950;
use bevy::color::palettes::tailwind::YELLOW_50;
use bevy::color::palettes::tailwind::YELLOW_100;
use bevy::color::palettes::tailwind::YELLOW_200;
use bevy::color::palettes::tailwind::YELLOW_300;
use bevy::color::palettes::tailwind::YELLOW_400;
use bevy::color::palettes::tailwind::YELLOW_500;
use bevy::color::palettes::tailwind::YELLOW_600;
use bevy::color::palettes::tailwind::YELLOW_700;
use bevy::color::palettes::tailwind::YELLOW_800;
use bevy::color::palettes::tailwind::YELLOW_900;
use bevy::color::palettes::tailwind::YELLOW_950;
use bevy::color::palettes::tailwind::ZINC_50;
use bevy::color::palettes::tailwind::ZINC_100;
use bevy::color::palettes::tailwind::ZINC_200;
use bevy::color::palettes::tailwind::ZINC_300;
use bevy::color::palettes::tailwind::ZINC_400;
use bevy::color::palettes::tailwind::ZINC_500;
use bevy::color::palettes::tailwind::ZINC_600;
use bevy::color::palettes::tailwind::ZINC_700;
use bevy::color::palettes::tailwind::ZINC_800;
use bevy::color::palettes::tailwind::ZINC_900;
use bevy::color::palettes::tailwind::ZINC_950;

pub const COLORS: [TailwindColor; 242] = [
    // slate
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V50,
        color: Color::Srgba(SLATE_50),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V100,
        color: Color::Srgba(SLATE_100),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V200,
        color: Color::Srgba(SLATE_200),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V300,
        color: Color::Srgba(SLATE_300),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V400,
        color: Color::Srgba(SLATE_400),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V500,
        color: Color::Srgba(SLATE_500),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V600,
        color: Color::Srgba(SLATE_600),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V700,
        color: Color::Srgba(SLATE_700),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V800,
        color: Color::Srgba(SLATE_800),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V900,
        color: Color::Srgba(SLATE_900),
    },
    TailwindColor {
        class: Class::Slate,
        variant: Variant::V950,
        color: Color::Srgba(SLATE_950),
    },
    // gray
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V50,
        color: Color::Srgba(GRAY_50),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V100,
        color: Color::Srgba(GRAY_100),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V200,
        color: Color::Srgba(GRAY_200),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V300,
        color: Color::Srgba(GRAY_300),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V400,
        color: Color::Srgba(GRAY_400),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V500,
        color: Color::Srgba(GRAY_500),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V600,
        color: Color::Srgba(GRAY_600),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V700,
        color: Color::Srgba(GRAY_700),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V800,
        color: Color::Srgba(GRAY_800),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V900,
        color: Color::Srgba(GRAY_900),
    },
    TailwindColor {
        class: Class::Gray,
        variant: Variant::V950,
        color: Color::Srgba(GRAY_950),
    },
    // zinc
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V50,
        color: Color::Srgba(ZINC_50),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V100,
        color: Color::Srgba(ZINC_100),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V200,
        color: Color::Srgba(ZINC_200),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V300,
        color: Color::Srgba(ZINC_300),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V400,
        color: Color::Srgba(ZINC_400),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V500,
        color: Color::Srgba(ZINC_500),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V600,
        color: Color::Srgba(ZINC_600),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V700,
        color: Color::Srgba(ZINC_700),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V800,
        color: Color::Srgba(ZINC_800),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V900,
        color: Color::Srgba(ZINC_900),
    },
    TailwindColor {
        class: Class::Zinc,
        variant: Variant::V950,
        color: Color::Srgba(ZINC_950),
    },
    // neutral
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V50,
        color: Color::Srgba(NEUTRAL_50),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V100,
        color: Color::Srgba(NEUTRAL_100),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V200,
        color: Color::Srgba(NEUTRAL_200),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V300,
        color: Color::Srgba(NEUTRAL_300),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V400,
        color: Color::Srgba(NEUTRAL_400),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V500,
        color: Color::Srgba(NEUTRAL_500),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V600,
        color: Color::Srgba(NEUTRAL_600),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V700,
        color: Color::Srgba(NEUTRAL_700),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V800,
        color: Color::Srgba(NEUTRAL_800),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V900,
        color: Color::Srgba(NEUTRAL_900),
    },
    TailwindColor {
        class: Class::Neutral,
        variant: Variant::V950,
        color: Color::Srgba(NEUTRAL_950),
    },
    // stone
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V50,
        color: Color::Srgba(STONE_50),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V100,
        color: Color::Srgba(STONE_100),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V200,
        color: Color::Srgba(STONE_200),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V300,
        color: Color::Srgba(STONE_300),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V400,
        color: Color::Srgba(STONE_400),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V500,
        color: Color::Srgba(STONE_500),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V600,
        color: Color::Srgba(STONE_600),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V700,
        color: Color::Srgba(STONE_700),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V800,
        color: Color::Srgba(STONE_800),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V900,
        color: Color::Srgba(STONE_900),
    },
    TailwindColor {
        class: Class::Stone,
        variant: Variant::V950,
        color: Color::Srgba(STONE_950),
    },
    // red
    TailwindColor {
        class: Class::Red,
        variant: Variant::V50,
        color: Color::Srgba(RED_50),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V100,
        color: Color::Srgba(RED_100),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V200,
        color: Color::Srgba(RED_200),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V300,
        color: Color::Srgba(RED_300),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V400,
        color: Color::Srgba(RED_400),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V500,
        color: Color::Srgba(RED_500),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V600,
        color: Color::Srgba(RED_600),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V700,
        color: Color::Srgba(RED_700),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V800,
        color: Color::Srgba(RED_800),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V900,
        color: Color::Srgba(RED_900),
    },
    TailwindColor {
        class: Class::Red,
        variant: Variant::V950,
        color: Color::Srgba(RED_950),
    },
    // orange
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V50,
        color: Color::Srgba(ORANGE_50),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V100,
        color: Color::Srgba(ORANGE_100),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V200,
        color: Color::Srgba(ORANGE_200),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V300,
        color: Color::Srgba(ORANGE_300),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V400,
        color: Color::Srgba(ORANGE_400),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V500,
        color: Color::Srgba(ORANGE_500),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V600,
        color: Color::Srgba(ORANGE_600),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V700,
        color: Color::Srgba(ORANGE_700),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V800,
        color: Color::Srgba(ORANGE_800),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V900,
        color: Color::Srgba(ORANGE_900),
    },
    TailwindColor {
        class: Class::Orange,
        variant: Variant::V950,
        color: Color::Srgba(ORANGE_950),
    },
    // amber
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V50,
        color: Color::Srgba(AMBER_50),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V100,
        color: Color::Srgba(AMBER_100),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V200,
        color: Color::Srgba(AMBER_200),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V300,
        color: Color::Srgba(AMBER_300),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V400,
        color: Color::Srgba(AMBER_400),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V500,
        color: Color::Srgba(AMBER_500),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V600,
        color: Color::Srgba(AMBER_600),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V700,
        color: Color::Srgba(AMBER_700),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V800,
        color: Color::Srgba(AMBER_800),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V900,
        color: Color::Srgba(AMBER_900),
    },
    TailwindColor {
        class: Class::Amber,
        variant: Variant::V950,
        color: Color::Srgba(AMBER_950),
    },
    // yellow
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V50,
        color: Color::Srgba(YELLOW_50),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V100,
        color: Color::Srgba(YELLOW_100),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V200,
        color: Color::Srgba(YELLOW_200),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V300,
        color: Color::Srgba(YELLOW_300),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V400,
        color: Color::Srgba(YELLOW_400),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V500,
        color: Color::Srgba(YELLOW_500),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V600,
        color: Color::Srgba(YELLOW_600),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V700,
        color: Color::Srgba(YELLOW_700),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V800,
        color: Color::Srgba(YELLOW_800),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V900,
        color: Color::Srgba(YELLOW_900),
    },
    TailwindColor {
        class: Class::Yellow,
        variant: Variant::V950,
        color: Color::Srgba(YELLOW_950),
    },
    // lime
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V50,
        color: Color::Srgba(LIME_50),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V100,
        color: Color::Srgba(LIME_100),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V200,
        color: Color::Srgba(LIME_200),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V300,
        color: Color::Srgba(LIME_300),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V400,
        color: Color::Srgba(LIME_400),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V500,
        color: Color::Srgba(LIME_500),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V600,
        color: Color::Srgba(LIME_600),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V700,
        color: Color::Srgba(LIME_700),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V800,
        color: Color::Srgba(LIME_800),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V900,
        color: Color::Srgba(LIME_900),
    },
    TailwindColor {
        class: Class::Lime,
        variant: Variant::V950,
        color: Color::Srgba(LIME_950),
    },
    // green
    TailwindColor {
        class: Class::Green,
        variant: Variant::V50,
        color: Color::Srgba(GREEN_50),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V100,
        color: Color::Srgba(GREEN_100),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V200,
        color: Color::Srgba(GREEN_200),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V300,
        color: Color::Srgba(GREEN_300),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V400,
        color: Color::Srgba(GREEN_400),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V500,
        color: Color::Srgba(GREEN_500),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V600,
        color: Color::Srgba(GREEN_600),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V700,
        color: Color::Srgba(GREEN_700),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V800,
        color: Color::Srgba(GREEN_800),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V900,
        color: Color::Srgba(GREEN_900),
    },
    TailwindColor {
        class: Class::Green,
        variant: Variant::V950,
        color: Color::Srgba(GREEN_950),
    },
    // emerald
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V50,
        color: Color::Srgba(EMERALD_50),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V100,
        color: Color::Srgba(EMERALD_100),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V200,
        color: Color::Srgba(EMERALD_200),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V300,
        color: Color::Srgba(EMERALD_300),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V400,
        color: Color::Srgba(EMERALD_400),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V500,
        color: Color::Srgba(EMERALD_500),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V600,
        color: Color::Srgba(EMERALD_600),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V700,
        color: Color::Srgba(EMERALD_700),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V800,
        color: Color::Srgba(EMERALD_800),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V900,
        color: Color::Srgba(EMERALD_900),
    },
    TailwindColor {
        class: Class::Emerald,
        variant: Variant::V950,
        color: Color::Srgba(EMERALD_950),
    },
    // teal
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V50,
        color: Color::Srgba(TEAL_50),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V100,
        color: Color::Srgba(TEAL_100),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V200,
        color: Color::Srgba(TEAL_200),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V300,
        color: Color::Srgba(TEAL_300),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V400,
        color: Color::Srgba(TEAL_400),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V500,
        color: Color::Srgba(TEAL_500),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V600,
        color: Color::Srgba(TEAL_600),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V700,
        color: Color::Srgba(TEAL_700),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V800,
        color: Color::Srgba(TEAL_800),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V900,
        color: Color::Srgba(TEAL_900),
    },
    TailwindColor {
        class: Class::Teal,
        variant: Variant::V950,
        color: Color::Srgba(TEAL_950),
    },
    // cyan
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V50,
        color: Color::Srgba(CYAN_50),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V100,
        color: Color::Srgba(CYAN_100),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V200,
        color: Color::Srgba(CYAN_200),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V300,
        color: Color::Srgba(CYAN_300),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V400,
        color: Color::Srgba(CYAN_400),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V500,
        color: Color::Srgba(CYAN_500),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V600,
        color: Color::Srgba(CYAN_600),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V700,
        color: Color::Srgba(CYAN_700),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V800,
        color: Color::Srgba(CYAN_800),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V900,
        color: Color::Srgba(CYAN_900),
    },
    TailwindColor {
        class: Class::Cyan,
        variant: Variant::V950,
        color: Color::Srgba(CYAN_950),
    },
    // sky
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V50,
        color: Color::Srgba(SKY_50),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V100,
        color: Color::Srgba(SKY_100),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V200,
        color: Color::Srgba(SKY_200),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V300,
        color: Color::Srgba(SKY_300),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V400,
        color: Color::Srgba(SKY_400),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V500,
        color: Color::Srgba(SKY_500),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V600,
        color: Color::Srgba(SKY_600),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V700,
        color: Color::Srgba(SKY_700),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V800,
        color: Color::Srgba(SKY_800),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V900,
        color: Color::Srgba(SKY_900),
    },
    TailwindColor {
        class: Class::Sky,
        variant: Variant::V950,
        color: Color::Srgba(SKY_950),
    },
    // blue
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V50,
        color: Color::Srgba(BLUE_50),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V100,
        color: Color::Srgba(BLUE_100),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V200,
        color: Color::Srgba(BLUE_200),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V300,
        color: Color::Srgba(BLUE_300),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V400,
        color: Color::Srgba(BLUE_400),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V500,
        color: Color::Srgba(BLUE_500),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V600,
        color: Color::Srgba(BLUE_600),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V700,
        color: Color::Srgba(BLUE_700),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V800,
        color: Color::Srgba(BLUE_800),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V900,
        color: Color::Srgba(BLUE_900),
    },
    TailwindColor {
        class: Class::Blue,
        variant: Variant::V950,
        color: Color::Srgba(BLUE_950),
    },
    // indigo
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V50,
        color: Color::Srgba(INDIGO_50),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V100,
        color: Color::Srgba(INDIGO_100),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V200,
        color: Color::Srgba(INDIGO_200),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V300,
        color: Color::Srgba(INDIGO_300),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V400,
        color: Color::Srgba(INDIGO_400),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V500,
        color: Color::Srgba(INDIGO_500),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V600,
        color: Color::Srgba(INDIGO_600),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V700,
        color: Color::Srgba(INDIGO_700),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V800,
        color: Color::Srgba(INDIGO_800),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V900,
        color: Color::Srgba(INDIGO_900),
    },
    TailwindColor {
        class: Class::Indigo,
        variant: Variant::V950,
        color: Color::Srgba(INDIGO_950),
    },
    // violet
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V50,
        color: Color::Srgba(VIOLET_50),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V100,
        color: Color::Srgba(VIOLET_100),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V200,
        color: Color::Srgba(VIOLET_200),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V300,
        color: Color::Srgba(VIOLET_300),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V400,
        color: Color::Srgba(VIOLET_400),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V500,
        color: Color::Srgba(VIOLET_500),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V600,
        color: Color::Srgba(VIOLET_600),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V700,
        color: Color::Srgba(VIOLET_700),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V800,
        color: Color::Srgba(VIOLET_800),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V900,
        color: Color::Srgba(VIOLET_900),
    },
    TailwindColor {
        class: Class::Violet,
        variant: Variant::V950,
        color: Color::Srgba(VIOLET_950),
    },
    // purple
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V50,
        color: Color::Srgba(PURPLE_50),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V100,
        color: Color::Srgba(PURPLE_100),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V200,
        color: Color::Srgba(PURPLE_200),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V300,
        color: Color::Srgba(PURPLE_300),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V400,
        color: Color::Srgba(PURPLE_400),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V500,
        color: Color::Srgba(PURPLE_500),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V600,
        color: Color::Srgba(PURPLE_600),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V700,
        color: Color::Srgba(PURPLE_700),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V800,
        color: Color::Srgba(PURPLE_800),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V900,
        color: Color::Srgba(PURPLE_900),
    },
    TailwindColor {
        class: Class::Purple,
        variant: Variant::V950,
        color: Color::Srgba(PURPLE_950),
    },
    // fuchsia
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V50,
        color: Color::Srgba(FUCHSIA_50),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V100,
        color: Color::Srgba(FUCHSIA_100),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V200,
        color: Color::Srgba(FUCHSIA_200),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V300,
        color: Color::Srgba(FUCHSIA_300),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V400,
        color: Color::Srgba(FUCHSIA_400),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V500,
        color: Color::Srgba(FUCHSIA_500),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V600,
        color: Color::Srgba(FUCHSIA_600),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V700,
        color: Color::Srgba(FUCHSIA_700),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V800,
        color: Color::Srgba(FUCHSIA_800),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V900,
        color: Color::Srgba(FUCHSIA_900),
    },
    TailwindColor {
        class: Class::Fuchsia,
        variant: Variant::V950,
        color: Color::Srgba(FUCHSIA_950),
    },
    // pink
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V50,
        color: Color::Srgba(PINK_50),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V100,
        color: Color::Srgba(PINK_100),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V200,
        color: Color::Srgba(PINK_200),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V300,
        color: Color::Srgba(PINK_300),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V400,
        color: Color::Srgba(PINK_400),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V500,
        color: Color::Srgba(PINK_500),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V600,
        color: Color::Srgba(PINK_600),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V700,
        color: Color::Srgba(PINK_700),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V800,
        color: Color::Srgba(PINK_800),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V900,
        color: Color::Srgba(PINK_900),
    },
    TailwindColor {
        class: Class::Pink,
        variant: Variant::V950,
        color: Color::Srgba(PINK_950),
    },
    // rose
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V50,
        color: Color::Srgba(ROSE_50),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V100,
        color: Color::Srgba(ROSE_100),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V200,
        color: Color::Srgba(ROSE_200),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V300,
        color: Color::Srgba(ROSE_300),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V400,
        color: Color::Srgba(ROSE_400),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V500,
        color: Color::Srgba(ROSE_500),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V600,
        color: Color::Srgba(ROSE_600),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V700,
        color: Color::Srgba(ROSE_700),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V800,
        color: Color::Srgba(ROSE_800),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V900,
        color: Color::Srgba(ROSE_900),
    },
    TailwindColor {
        class: Class::Rose,
        variant: Variant::V950,
        color: Color::Srgba(ROSE_950),
    },
];

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TailwindColor {
    pub class: Class,
    pub variant: Variant,
    pub color: Color,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Class {
    Slate,
    Gray,
    Zinc,
    Neutral,
    Stone,
    Red,
    Orange,
    Amber,
    Yellow,
    Lime,
    Green,
    Emerald,
    Teal,
    Cyan,
    Sky,
    Blue,
    Indigo,
    Violet,
    Purple,
    Fuchsia,
    Pink,
    Rose,
}
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Variant {
    V50,
    V100,
    V200,
    V300,
    V400,
    V500,
    V600,
    V700,
    V800,
    V900,
    V950,
}
