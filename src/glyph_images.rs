//! The images the widgets draw themselves (the combo chevron, the stepper's minus and plus, the
//! light / dark toggle's icons), generated once per app and shared by every instance.

#![expect(
    clippy::redundant_pub_crate,
    reason = "the module is crate-private, while its items keep pub(crate) to say how far they reach (pub would trip unreachable_pub), as in focus.rs"
)]

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

/// The handles of the generated images made so far in this app, kept alive here so the images
/// outlive the widgets that show them. A key is added only once the app has an
/// `Assets<Image>`, so an app that adds it late gets its images then.
#[derive(Resource, Debug, Default)]
pub(crate) struct GlyphImages {
    handles: HashMap<GlyphKey, Handle<Image>>,
}

/// The shared image for `key`, made with `generate` the first time it is asked for (or again
/// when the app removed it from `Assets<Image>`). With no `Assets<Image>` the default handle is
/// returned and nothing is remembered; without the `GlyphImages` resource (the app did not add
/// `WidgetsPlugin`) the image is made per call, as before sharing.
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
