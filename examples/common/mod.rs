//! A small icon drawn at startup, shared by the examples that need an image for an icon button,
//! and the `bsn!` entry that inserts a widget holding it.

#![expect(
    unreachable_pub,
    reason = "`pub` is the only visibility clippy accepts in a private example module"
)]

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// A white cross on a transparent 48 by 48 image, for the icon of a button.
pub fn cross_icon(images: &mut Assets<Image>) -> Handle<Image> {
    const SIZE: u32 = 48;
    let size = SIZE as f32;
    let mut data = vec![0u8; (SIZE * SIZE * 4) as usize];
    for y in 0..SIZE {
        for x in 0..SIZE {
            let (u, v) = (x as f32 + 0.5, y as f32 + 0.5);
            let inside = (0.22 * size..=0.78 * size).contains(&u)
                && (0.22 * size..=0.78 * size).contains(&v);
            let on_diagonal = (u - v).abs() < 0.07 * size || (u + v - size).abs() < 0.07 * size;
            if inside && on_diagonal {
                let at = ((y * SIZE + x) * 4) as usize;
                if let Some(pixel) = data.get_mut(at..at + 4) {
                    pixel.copy_from_slice(&[255; 4]);
                }
            }
        }
    }
    images.add(Image::new(
        Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}
