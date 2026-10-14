//! All Tailwind colours, for colour pickers.
//!
//! The theme itself uses [`bevy::color::palettes::tailwind`] directly.

use bevy::color::palettes::tailwind::{
    AMBER_50, AMBER_100, AMBER_200, AMBER_300, AMBER_400, AMBER_500, AMBER_600, AMBER_700,
    AMBER_800, AMBER_900, AMBER_950, BLUE_50, BLUE_100, BLUE_200, BLUE_300, BLUE_400, BLUE_500,
    BLUE_600, BLUE_700, BLUE_800, BLUE_900, BLUE_950, CYAN_50, CYAN_100, CYAN_200, CYAN_300,
    CYAN_400, CYAN_500, CYAN_600, CYAN_700, CYAN_800, CYAN_900, CYAN_950, EMERALD_50, EMERALD_100,
    EMERALD_200, EMERALD_300, EMERALD_400, EMERALD_500, EMERALD_600, EMERALD_700, EMERALD_800,
    EMERALD_900, EMERALD_950, FUCHSIA_50, FUCHSIA_100, FUCHSIA_200, FUCHSIA_300, FUCHSIA_400,
    FUCHSIA_500, FUCHSIA_600, FUCHSIA_700, FUCHSIA_800, FUCHSIA_900, FUCHSIA_950, GRAY_50,
    GRAY_100, GRAY_200, GRAY_300, GRAY_400, GRAY_500, GRAY_600, GRAY_700, GRAY_800, GRAY_900,
    GRAY_950, GREEN_50, GREEN_100, GREEN_200, GREEN_300, GREEN_400, GREEN_500, GREEN_600,
    GREEN_700, GREEN_800, GREEN_900, GREEN_950, INDIGO_50, INDIGO_100, INDIGO_200, INDIGO_300,
    INDIGO_400, INDIGO_500, INDIGO_600, INDIGO_700, INDIGO_800, INDIGO_900, INDIGO_950, LIME_50,
    LIME_100, LIME_200, LIME_300, LIME_400, LIME_500, LIME_600, LIME_700, LIME_800, LIME_900,
    LIME_950, NEUTRAL_50, NEUTRAL_100, NEUTRAL_200, NEUTRAL_300, NEUTRAL_400, NEUTRAL_500,
    NEUTRAL_600, NEUTRAL_700, NEUTRAL_800, NEUTRAL_900, NEUTRAL_950, ORANGE_50, ORANGE_100,
    ORANGE_200, ORANGE_300, ORANGE_400, ORANGE_500, ORANGE_600, ORANGE_700, ORANGE_800, ORANGE_900,
    ORANGE_950, PINK_50, PINK_100, PINK_200, PINK_300, PINK_400, PINK_500, PINK_600, PINK_700,
    PINK_800, PINK_900, PINK_950, PURPLE_50, PURPLE_100, PURPLE_200, PURPLE_300, PURPLE_400,
    PURPLE_500, PURPLE_600, PURPLE_700, PURPLE_800, PURPLE_900, PURPLE_950, RED_50, RED_100,
    RED_200, RED_300, RED_400, RED_500, RED_600, RED_700, RED_800, RED_900, RED_950, ROSE_50,
    ROSE_100, ROSE_200, ROSE_300, ROSE_400, ROSE_500, ROSE_600, ROSE_700, ROSE_800, ROSE_900,
    ROSE_950, SKY_50, SKY_100, SKY_200, SKY_300, SKY_400, SKY_500, SKY_600, SKY_700, SKY_800,
    SKY_900, SKY_950, SLATE_50, SLATE_100, SLATE_200, SLATE_300, SLATE_400, SLATE_500, SLATE_600,
    SLATE_700, SLATE_800, SLATE_900, SLATE_950, STONE_50, STONE_100, STONE_200, STONE_300,
    STONE_400, STONE_500, STONE_600, STONE_700, STONE_800, STONE_900, STONE_950, TEAL_50, TEAL_100,
    TEAL_200, TEAL_300, TEAL_400, TEAL_500, TEAL_600, TEAL_700, TEAL_800, TEAL_900, TEAL_950,
    VIOLET_50, VIOLET_100, VIOLET_200, VIOLET_300, VIOLET_400, VIOLET_500, VIOLET_600, VIOLET_700,
    VIOLET_800, VIOLET_900, VIOLET_950, YELLOW_50, YELLOW_100, YELLOW_200, YELLOW_300, YELLOW_400,
    YELLOW_500, YELLOW_600, YELLOW_700, YELLOW_800, YELLOW_900, YELLOW_950, ZINC_50, ZINC_100,
    ZINC_200, ZINC_300, ZINC_400, ZINC_500, ZINC_600, ZINC_700, ZINC_800, ZINC_900, ZINC_950,
};
use bevy::prelude::*;

/// All 242 Tailwind colours: 22 classes with 11 variants each.
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

/// One Tailwind colour.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TailwindColor {
    /// The hue, such as `Blue`.
    pub class: Class,
    /// The shade, from 50 (lightest) to 950 (darkest).
    pub variant: Variant,
    /// The colour itself.
    pub color: Color,
}
/// The hue of a Tailwind colour.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Class {
    /// Tailwind `slate`.
    Slate,
    /// Tailwind `gray`.
    Gray,
    /// Tailwind `zinc`.
    Zinc,
    /// Tailwind `neutral`.
    Neutral,
    /// Tailwind `stone`.
    Stone,
    /// Tailwind `red`.
    Red,
    /// Tailwind `orange`.
    Orange,
    /// Tailwind `amber`.
    Amber,
    /// Tailwind `yellow`.
    Yellow,
    /// Tailwind `lime`.
    Lime,
    /// Tailwind `green`.
    Green,
    /// Tailwind `emerald`.
    Emerald,
    /// Tailwind `teal`.
    Teal,
    /// Tailwind `cyan`.
    Cyan,
    /// Tailwind `sky`.
    Sky,
    /// Tailwind `blue`.
    Blue,
    /// Tailwind `indigo`.
    Indigo,
    /// Tailwind `violet`.
    Violet,
    /// Tailwind `purple`.
    Purple,
    /// Tailwind `fuchsia`.
    Fuchsia,
    /// Tailwind `pink`.
    Pink,
    /// Tailwind `rose`.
    Rose,
}
/// The shade of a Tailwind colour.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Variant {
    /// Shade 50.
    V50,
    /// Shade 100.
    V100,
    /// Shade 200.
    V200,
    /// Shade 300.
    V300,
    /// Shade 400.
    V400,
    /// Shade 500.
    V500,
    /// Shade 600.
    V600,
    /// Shade 700.
    V700,
    /// Shade 800.
    V800,
    /// Shade 900.
    V900,
    /// Shade 950.
    V950,
}

/// The accessible name of a colour: its Tailwind name (`blue-500`) when it is
/// one of [`COLORS`], else its sRGB hex (`#3b82f6`, with the alpha appended
/// only when it is not 1).
pub(crate) fn color_name(color: Color) -> String {
    COLORS
        .iter()
        .find(|tailwind| tailwind.color == color)
        .map_or_else(
            || Srgba::from(color).to_hex().to_lowercase(),
            |tailwind| {
                let class = format!("{:?}", tailwind.class).to_lowercase();
                let variant = format!("{:?}", tailwind.variant);
                format!("{class}-{}", variant.trim_start_matches('V'))
            },
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tailwind_colour_is_named_after_its_class_and_shade() {
        assert_eq!(color_name(Color::from(BLUE_500)), "blue-500");
        assert_eq!(color_name(Color::from(SLATE_50)), "slate-50");
    }

    #[test]
    fn a_colour_outside_tailwind_is_named_by_its_hex() {
        assert_eq!(color_name(Color::srgb_u8(0x12, 0xab, 0xef)), "#12abef");
    }

    #[test]
    fn the_alpha_of_a_colour_is_appended_only_when_not_one() {
        assert_eq!(
            color_name(Color::srgba_u8(0x12, 0xab, 0xef, 0x80)),
            "#12abef80"
        );
    }
}
