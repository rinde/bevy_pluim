//! Small white-on-transparent icons drawn at startup, so the example ships no image files.

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// A point of an icon in the unit square, with the width over the height of the image.
#[derive(Clone, Copy)]
struct Point {
    u: f32,
    v: f32,
    aspect: f32,
}

fn rect(p: Point, u0: f32, u1: f32, v0: f32, v1: f32) -> bool {
    (u0..=u1).contains(&p.u) && (v0..=v1).contains(&p.v)
}

fn disc(p: Point, center_u: f32, center_v: f32, radius: f32) -> bool {
    let du = (p.u - center_u) * p.aspect;
    let dv = p.v - center_v;
    dv.mul_add(dv, du * du) <= radius * radius
}

fn ring(p: Point, center_u: f32, center_v: f32, outer: f32, inner: f32) -> bool {
    disc(p, center_u, center_v, outer) && !disc(p, center_u, center_v, inner)
}

/// Which side of the line `from`-`to` the point is on.
fn side(p: Point, from: (f32, f32), to: (f32, f32)) -> f32 {
    let (dx, dy) = (p.u - to.0, p.v - to.1);
    let (ex, ey) = (from.0 - to.0, from.1 - to.1);
    dx.mul_add(ey, -(ex * dy))
}

fn triangle(p: Point, a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> bool {
    let signs = [side(p, a, b), side(p, b, c), side(p, c, a)];
    signs.iter().all(|s| *s >= 0.0) || signs.iter().all(|s| *s <= 0.0)
}

fn cross(p: Point) -> bool {
    rect(p, 0.22, 0.78, 0.22, 0.78) && ((p.u - p.v).abs() < 0.07 || (p.u + p.v - 1.0).abs() < 0.07)
}

fn pause(p: Point) -> bool {
    rect(p, 0.27, 0.43, 0.22, 0.78) || rect(p, 0.57, 0.73, 0.22, 0.78)
}

fn bell(p: Point) -> bool {
    disc(p, 0.5, 0.42, 0.28) && p.v < 0.55
        || rect(p, 0.22, 0.78, 0.4, 0.68)
        || rect(p, 0.14, 0.86, 0.64, 0.72)
        || disc(p, 0.5, 0.83, 0.08)
}

fn train(p: Point) -> bool {
    rect(p, 0.2, 0.8, 0.15, 0.72) && !rect(p, 0.3, 0.7, 0.26, 0.48)
        || disc(p, 0.34, 0.84, 0.08)
        || disc(p, 0.66, 0.84, 0.08)
}

fn stop(p: Point) -> bool {
    ring(p, 0.5, 0.5, 0.4, 0.28) || disc(p, 0.5, 0.5, 0.12)
}

fn schedule(p: Point) -> bool {
    ring(p, 0.5, 0.5, 0.4, 0.31)
        || rect(p, 0.47, 0.53, 0.22, 0.52)
        || rect(p, 0.47, 0.7, 0.47, 0.53)
}

fn preferences(p: Point) -> bool {
    [(0.25, 0.35), (0.5, 0.65), (0.75, 0.45)]
        .iter()
        .any(|(v, knob)| rect(p, 0.14, 0.86, v - 0.03, v + 0.03) || disc(p, *knob, *v, 0.1))
}

fn select(p: Point) -> bool {
    triangle(p, (0.28, 0.12), (0.28, 0.82), (0.78, 0.58))
}

fn rail(p: Point) -> bool {
    rect(p, 0.26, 0.33, 0.08, 0.92)
        || rect(p, 0.67, 0.74, 0.08, 0.92)
        || [0.2, 0.4, 0.6, 0.8]
            .iter()
            .any(|v| rect(p, 0.3, 0.7, v - 0.035, v + 0.035))
}

fn signal(p: Point) -> bool {
    (rect(p, 0.46, 0.54, 0.45, 0.95) || rect(p, 0.28, 0.72, 0.06, 0.45))
        && !disc(p, 0.4, 0.26, 0.07)
        && !disc(p, 0.6, 0.26, 0.07)
}

fn locomotive(p: Point) -> bool {
    rect(p, 0.1, 0.72, 0.32, 0.68)
        || rect(p, 0.14, 0.24, 0.12, 0.34)
        || rect(p, 0.72, 0.9, 0.2, 0.68)
        || [0.25, 0.45, 0.7].iter().any(|u| disc(p, *u, 0.78, 0.12))
}

fn boxcar(p: Point) -> bool {
    rect(p, 0.1, 0.9, 0.2, 0.68) || disc(p, 0.28, 0.79, 0.11) || disc(p, 0.72, 0.79, 0.11)
}

fn flatcar(p: Point) -> bool {
    rect(p, 0.1, 0.9, 0.5, 0.64) || disc(p, 0.28, 0.77, 0.11) || disc(p, 0.72, 0.77, 0.11)
}

/// The handles of the icons the HUD uses.
#[derive(Resource, Clone)]
pub(super) struct Icons {
    pub(super) pause: Handle<Image>,
    pub(super) bell: Handle<Image>,
    pub(super) train: Handle<Image>,
    pub(super) stop: Handle<Image>,
    pub(super) schedule: Handle<Image>,
    pub(super) preferences: Handle<Image>,
    pub(super) select: Handle<Image>,
    pub(super) rail: Handle<Image>,
    pub(super) signal: Handle<Image>,
    pub(super) bulldoze: Handle<Image>,
    pub(super) close: Handle<Image>,
    pub(super) locomotive: Handle<Image>,
    pub(super) boxcar: Handle<Image>,
    pub(super) flatcar: Handle<Image>,
}

fn draw(
    images: &mut Assets<Image>,
    width: u32,
    height: u32,
    shape: fn(Point) -> bool,
) -> Handle<Image> {
    let mut data = vec![0u8; (width * height * 4) as usize];
    for y in 0..height {
        for x in 0..width {
            let p = Point {
                u: (x as f32 + 0.5) / width as f32,
                v: (y as f32 + 0.5) / height as f32,
                aspect: width as f32 / height as f32,
            };
            if shape(p) {
                let at = ((y * width + x) * 4) as usize;
                if let Some(pixel) = data.get_mut(at..at + 4) {
                    pixel.copy_from_slice(&[255; 4]);
                }
            }
        }
    }
    images.add(Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ))
}

impl Icons {
    /// Draws every icon into `images`.
    pub(super) fn draw(images: &mut Assets<Image>) -> Self {
        let mut square = |shape| draw(images, 48, 48, shape);
        let pause = square(pause);
        let bell = square(bell);
        let train = square(train);
        let stop = square(stop);
        let schedule = square(schedule);
        let preferences = square(preferences);
        let select = square(select);
        let rail = square(rail);
        let signal = square(signal);
        let bulldoze = square(cross);
        let close = square(cross);
        Self {
            pause,
            bell,
            train,
            stop,
            schedule,
            preferences,
            select,
            rail,
            signal,
            bulldoze,
            close,
            locomotive: draw(images, 144, 60, locomotive),
            boxcar: draw(images, 144, 60, boxcar),
            flatcar: draw(images, 144, 60, flatcar),
        }
    }
}
