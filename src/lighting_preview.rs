//! @file lighting_preview.rs
//! @brief Animated visual illustrations of the eight reference lighting modes.
//! @author modolet <y@xxyx.io>
//! @date 2026-10-12

use crate::layout::Key;
use std::f32::consts::TAU;

/// This is a visual illustration, not a simulation of undocumented firmware.
/// Ripple holds its origin and start time in the same seconds clock as the preview.
pub fn color(
    effect: u8,
    base: [u8; 3],
    key: &Key,
    seconds: f32,
    ripple: Option<(f32, f32, f32)>,
) -> [u8; 3] {
    let x = key.x + key.width / 2.;
    let row = f32::from(key.row);
    let brightness = match effect {
        1 => return base,
        2 => 0.12 + 0.88 * (0.5 + 0.5 * (seconds * TAU / 2.8).cos()),
        3 => return rainbow(seconds / 4.),
        4 => {
            let cycle = (seconds * 1.6 + x * 0.37).rem_euclid(5.5);
            (1. - (row - cycle).abs()).max(0.)
        }
        5 => {
            let (origin_x, origin_row, started) =
                ripple.unwrap_or((8., 2., seconds - seconds.rem_euclid(3.)));
            let distance = ((x - origin_x).powi(2) + (row - origin_row).powi(2)).sqrt();
            let radius = (seconds - started) * 6.;
            (1. - (distance - radius).abs() / 1.5).max(0.)
        }
        6 => return rainbow(x / 16. - seconds / 3.),
        7 => (0.5 + 0.5 * (seconds * 2.2 + f32::from(key.id) * 1.618).sin()).powi(12),
        8 => {
            let distance = (x - seconds * 5.).rem_euclid(16.);
            0.08 + 0.92 * (1. - distance.min(16. - distance) / 3.).max(0.)
        }
        _ => return base,
    };
    base.map(|channel| (f32::from(channel) * brightness.clamp(0., 1.)).round() as u8)
}

fn rainbow(hue: f32) -> [u8; 3] {
    let position = hue.rem_euclid(1.) * 6.;
    let rising = 0.18 + 0.82 * position.fract();
    let falling = 1.18 - rising;
    let rgb = match position as u8 {
        0 => [1., rising, 0.18],
        1 => [falling, 1., 0.18],
        2 => [0.18, 1., rising],
        3 => [0.18, falling, 1.],
        4 => [rising, 0.18, 1.],
        _ => [1., 0.18, falling],
    };
    rgb.map(|channel| (channel * 255.).round() as u8)
}
