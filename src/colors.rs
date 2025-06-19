use bevy::prelude::Color;

// =============================================================================
// Gray Palette
// =============================================================================

/// Gray 50 - Lightest gray, used for subtle backgrounds
pub const GRAY_50: Color = Color::srgb(0.97, 0.97, 0.98); // #F9FAFB

/// Gray 100 - Very light gray
pub const GRAY_100: Color = Color::srgb(0.95, 0.95, 0.96); // #F3F4F6

/// Gray 200 - Light gray, good for borders and dividers
pub const GRAY_200: Color = Color::srgb(0.91, 0.91, 0.93); // #E5E7EB

/// Gray 300 - Medium-light gray, primary border color
pub const GRAY_300: Color = Color::srgb(0.83, 0.84, 0.86); // #D1D5DB

/// Gray 400 - Medium gray, hover states and secondary borders
pub const GRAY_400: Color = Color::srgb(0.75, 0.76, 0.78); // #9CA3AF

/// Gray 500 - Medium gray, secondary text and icons
pub const GRAY_500: Color = Color::srgb(0.55, 0.55, 0.58); // #6B7280

/// Gray 600 - Medium-dark gray
pub const GRAY_600: Color = Color::srgb(0.33, 0.34, 0.36); // #4B5563

/// Gray 700 - Dark gray
pub const GRAY_700: Color = Color::srgb(0.24, 0.25, 0.27); // #374151

/// Gray 800 - Very dark gray
pub const GRAY_800: Color = Color::srgb(0.16, 0.16, 0.18); // #1F2937

/// Gray 900 - Darkest gray, primary text color
pub const GRAY_900: Color = Color::srgb(0.11, 0.11, 0.13); // #111827

// =============================================================================
// Blue Palette (Primary/Accent Colors)
// =============================================================================

/// Blue 50 - Lightest blue background
pub const BLUE_50: Color = Color::srgb(0.93, 0.96, 1.0); // #EBF8FF

/// Blue 100 - Very light blue
pub const BLUE_100: Color = Color::srgb(0.86, 0.93, 1.0); // #DBEAFE

/// Blue 200 - Light blue
pub const BLUE_200: Color = Color::srgb(0.75, 0.87, 1.0); // #BFDBFE

/// Blue 300 - Medium-light blue
pub const BLUE_300: Color = Color::srgb(0.58, 0.77, 0.99); // #93C5FD

/// Blue 400 - Medium blue
pub const BLUE_400: Color = Color::srgb(0.38, 0.64, 0.98); // #60A5FA

/// Blue 500 - Primary blue, main accent color
pub const BLUE_500: Color = Color::srgb(0.24, 0.51, 0.98); // #3B82F6

/// Blue 600 - Medium-dark blue, primary button color
pub const BLUE_600: Color = Color::srgb(0.15, 0.43, 0.92); // #2563EB

/// Blue 700 - Dark blue
pub const BLUE_700: Color = Color::srgb(0.11, 0.35, 0.8); // #1D4ED8

/// Blue 800 - Very dark blue
pub const BLUE_800: Color = Color::srgb(0.12, 0.27, 0.67); // #1E40AF

/// Blue 900 - Darkest blue
pub const BLUE_900: Color = Color::srgb(0.12, 0.20, 0.53); // #1E3A8A

// =============================================================================
// Green Palette (Success States)
// =============================================================================

/// Green 50 - Lightest green background
pub const GREEN_50: Color = Color::srgb(0.93, 0.98, 0.94); // #ECFDF5

/// Green 100 - Very light green
pub const GREEN_100: Color = Color::srgb(0.86, 0.95, 0.88); // #D1FAE5

/// Green 500 - Primary green, success color
pub const GREEN_500: Color = Color::srgb(0.13, 0.69, 0.30); // #10B981

/// Green 600 - Medium-dark green
pub const GREEN_600: Color = Color::srgb(0.02, 0.58, 0.27); // #059669

/// Green 700 - Dark green
pub const GREEN_700: Color = Color::srgb(0.01, 0.46, 0.20); // #047857

// =============================================================================
// Red Palette (Error/Danger States)
// =============================================================================

/// Red 50 - Lightest red background
pub const RED_50: Color = Color::srgb(0.99, 0.95, 0.95); // #FEF2F2

/// Red 100 - Very light red
pub const RED_100: Color = Color::srgb(0.99, 0.89, 0.89); // #FEE2E2

/// Red 500 - Primary red, error color
pub const RED_500: Color = Color::srgb(0.94, 0.26, 0.21); // #EF4444

/// Red 600 - Medium-dark red
pub const RED_600: Color = Color::srgb(0.86, 0.15, 0.14); // #DC2626

/// Red 700 - Dark red
pub const RED_700: Color = Color::srgb(0.73, 0.08, 0.08); // #B91C1C

// =============================================================================
// Yellow/Orange Palette (Warning States)
// =============================================================================

/// Yellow 50 - Lightest yellow background
pub const YELLOW_50: Color = Color::srgb(1.0, 0.99, 0.93); // #FFFBEB

/// Yellow 100 - Very light yellow
pub const YELLOW_100: Color = Color::srgb(1.0, 0.95, 0.82); // #FEF3CD

/// Yellow 500 - Primary yellow, warning color
pub const YELLOW_500: Color = Color::srgb(0.96, 0.74, 0.06); // #F59E0B

/// Yellow 600 - Medium-dark yellow
pub const YELLOW_600: Color = Color::srgb(0.85, 0.62, 0.02); // #D97706

/// Orange 500 - Primary orange
pub const ORANGE_500: Color = Color::srgb(0.98, 0.45, 0.13); // #F97316

/// Orange 600 - Medium-dark orange
pub const ORANGE_600: Color = Color::srgb(0.92, 0.35, 0.06); // #EA580C

// =============================================================================
// Purple Palette (Secondary/Info Colors)
// =============================================================================

/// Purple 50 - Lightest purple background
pub const PURPLE_50: Color = Color::srgb(0.98, 0.95, 1.0); // #FAF5FF

/// Purple 100 - Very light purple
pub const PURPLE_100: Color = Color::srgb(0.95, 0.91, 1.0); // #F3E8FF

/// Purple 500 - Primary purple
pub const PURPLE_500: Color = Color::srgb(0.55, 0.31, 0.86); // #8B5CF6

/// Purple 600 - Medium-dark purple
pub const PURPLE_600: Color = Color::srgb(0.48, 0.22, 0.82); // #7C3AED

// =============================================================================
// Base Colors
// =============================================================================

/// Pure white
pub const WHITE: Color = Color::WHITE; // #FFFFFF

/// Pure black
pub const BLACK: Color = Color::BLACK; // #000000

/// Transparent color
pub const TRANSPARENT: Color = Color::NONE;

// =============================================================================
// Shadow and Overlay Colors
// =============================================================================

/// Small shadow - subtle depth
pub const SHADOW_SM: Color = Color::srgba(0.0, 0.0, 0.0, 0.05);

/// Medium shadow - standard depth for dropdowns and cards
pub const SHADOW_MD: Color = Color::srgba(0.0, 0.0, 0.0, 0.1);

/// Large shadow - prominent depth for modals
pub const SHADOW_LG: Color = Color::srgba(0.0, 0.0, 0.0, 0.15);

/// Extra large shadow - dramatic depth
pub const SHADOW_XL: Color = Color::srgba(0.0, 0.0, 0.0, 0.25);

/// Modal overlay background
pub const OVERLAY: Color = Color::srgba(0.0, 0.0, 0.0, 0.5);

/// Light overlay for subtle backgrounds
pub const OVERLAY_LIGHT: Color = Color::srgba(0.0, 0.0, 0.0, 0.1);

// =============================================================================
// Semantic Color Aliases
// =============================================================================

/// Primary brand color (maps to Blue 600)
pub const PRIMARY: Color = BLUE_600;

/// Primary hover state
pub const PRIMARY_HOVER: Color = BLUE_700;

/// Secondary color (maps to Gray 600)
pub const SECONDARY: Color = GRAY_600;

/// Success color (maps to Green 500)
pub const SUCCESS: Color = GREEN_500;

/// Warning color (maps to Yellow 500)
pub const WARNING: Color = YELLOW_500;

/// Error/Danger color (maps to Red 500)
pub const DANGER: Color = RED_500;

/// Info color (maps to Blue 500)
pub const INFO: Color = BLUE_500;

// =============================================================================
// Text Color Aliases
// =============================================================================

/// Primary text color - highest contrast
pub const TEXT_PRIMARY: Color = GRAY_900;

/// Secondary text color - medium contrast
pub const TEXT_SECONDARY: Color = GRAY_600;

/// Muted text color - low contrast
pub const TEXT_MUTED: Color = GRAY_500;

/// Disabled text color - very low contrast
pub const TEXT_DISABLED: Color = GRAY_400;

/// Text on dark backgrounds
pub const TEXT_ON_DARK: Color = WHITE;

// =============================================================================
// Background Color Aliases
// =============================================================================

/// Primary background color
pub const BG_PRIMARY: Color = WHITE;

/// Secondary background color
pub const BG_SECONDARY: Color = GRAY_50;

/// Muted background color
pub const BG_MUTED: Color = GRAY_100;

/// Dark background color
pub const BG_DARK: Color = GRAY_900;

// =============================================================================
// Border Color Aliases
// =============================================================================

/// Primary border color
pub const BORDER_PRIMARY: Color = GRAY_300;

/// Secondary border color
pub const BORDER_SECONDARY: Color = GRAY_200;

/// Muted border color
pub const BORDER_MUTED: Color = GRAY_100;

/// Focus border color
pub const BORDER_FOCUS: Color = BLUE_500;

/// Error border color
pub const BORDER_ERROR: Color = RED_500;
