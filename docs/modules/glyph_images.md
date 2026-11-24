# `glyph_images.rs`

Crate-private per-app cache of the images the library draws (chevron, minus, plus, sun, computer, moon).

Read this before changing `src/glyph_images.rs`. When the module's behaviour changes, update this file in the same PR.

## Details

crate-private: `GlyphImages`, a per-app resource (initialised by `WidgetsPlugin`, so independent `App`s keep their own) caching one `Handle<Image>` per `GlyphKey` (`Chevron`, `Minus`, `Plus`, `Sun`, `Computer`, `Moon`), and `shared(world, key, generate)`, which `Combo`, `FilterCombo`, a `NumericStepper` left at its default icons and `LightDarkToggle` use instead of adding an image per instance: many widgets show 6 images in all. Without `Assets<Image>` it returns `Handle::default()` and caches nothing, so a widget made before the assets exist gets images only when inserted again after; an image the app removed from `Assets<Image>` is generated again; the cache holds a strong handle, so the images live as long as the app. A custom stepper icon is never touched (`many_widgets_share_one_image_per_generated_glyph`, `a_custom_stepper_icon_is_kept_and_only_the_default_one_is_shared`, `an_app_without_image_assets_gets_default_handles_and_the_images_once_assets_exist`, `separate_apps_do_not_share_glyph_images`, `an_image_the_app_removed_is_generated_again`)
