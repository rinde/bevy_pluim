//! The images the widgets draw themselves, generated once per app.

#![expect(clippy::redundant_pub_crate, reason = "pub would trip unreachable_pub")]

use bevy::ecs::world::DeferredWorld;
use bevy::platform::collections::HashMap;
use bevy::prelude::*;

/// Which generated image a widget wants.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub(crate) enum GlyphKey {
    /// The triangle of `Combo` and `FilterCombo`.
    Chevron,
    /// The `NumericStepper`'s default minus.
    Minus,
    /// The `NumericStepper`'s default plus.
    Plus,
    /// The `LightDarkToggle`'s sun.
    Sun,
    /// The `LightDarkToggle`'s computer.
    Computer,
    /// The `LightDarkToggle`'s moon.
    Moon,
}

/// The handles of the generated images, kept alive so they outlive the widgets.
#[derive(Resource, Debug, Default)]
pub(crate) struct GlyphImages {
    handles: HashMap<GlyphKey, Handle<Image>>,
}

/// The shared image for `key`, made with `generate` on first use or after it left `Assets<Image>`.
pub(crate) fn shared(
    world: &mut DeferredWorld,
    key: GlyphKey,
    generate: impl FnOnce() -> Image,
) -> Handle<Image> {
    let known = world
        .get_resource::<GlyphImages>()
        .and_then(|glyphs| glyphs.handles.get(&key).cloned());
    let Some(mut images) = world.get_resource_mut::<Assets<Image>>() else {
        return Handle::default();
    };
    if let Some(handle) = known
        && images.contains(&handle)
    {
        return handle;
    }
    let handle = images.add(generate());
    if let Some(mut glyphs) = world.get_resource_mut::<GlyphImages>() {
        glyphs.handles.insert(key, handle.clone());
    }
    handle
}
