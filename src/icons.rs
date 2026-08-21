//! Original weather glyphs as declarative shape groups (docs/shapes.md): each `weather_icon`
//! flattens its shapes into ONE canvas leaf — no bundled assets, so they render identically on
//! every backend and scale cleanly. Geometry is authored in fractional [0,1] coordinates of the
//! square via `.at`; stroke widths are points derived from the known `size`.

use crate::weather::Family;
use day::prelude::*;

/// Which glyph to draw. Derived from a `Family` plus day/night for clear & partly-cloudy skies.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    Sun,
    Moon,
    PartlyDay,
    PartlyNight,
    Cloud,
    Fog,
    Drizzle,
    Rain,
    Snow,
    Thunder,
}

impl Glyph {
    pub fn of(family: Family, is_day: bool) -> Glyph {
        match family {
            Family::Clear => {
                if is_day {
                    Glyph::Sun
                } else {
                    Glyph::Moon
                }
            }
            Family::PartlyCloudy => {
                if is_day {
                    Glyph::PartlyDay
                } else {
                    Glyph::PartlyNight
                }
            }
            Family::Cloudy => Glyph::Cloud,
            Family::Fog => Glyph::Fog,
            Family::Drizzle => Glyph::Drizzle,
            Family::Rain => Glyph::Rain,
            Family::Snow => Glyph::Snow,
            Family::Thunder => Glyph::Thunder,
        }
    }
}

// Palette (warm sun, pale moon, soft clouds, cool precipitation) — reads well on the sky tints.
const SUN: Color = Color::hex(0xFFD24A);
const SUN_CORE: Color = Color::hex(0xFFB300);
const MOON: Color = Color::hex(0xF3EDD2);
const CLOUD: Color = Color::hex(0xF2F6FB);
const CLOUD_DK: Color = Color::hex(0xCAD5E2);
const RAINDROP: Color = Color::hex(0x8FC7FF);
const SNOW: Color = Color::hex(0xFFFFFF);
const BOLT: Color = Color::hex(0xFFD24A);
const FOGLINE: Color = Color::hex(0xE3E9F0);

/// A weather glyph as a square shape group — one canvas leaf sized to `size`.
pub fn weather_icon(glyph: Glyph, size: f64) -> impl Piece {
    shape_group(glyph_shapes(glyph, size)).frame(size, size)
}

/// The glyph's shapes, in draw order.
fn glyph_shapes(glyph: Glyph, size: f64) -> Vec<ShapePiece> {
    match glyph {
        Glyph::Sun => sun(size, 0.5, 0.5, 1.0),
        Glyph::Moon => moon(),
        Glyph::Cloud => cloud(0.06, 0.20, 0.88, 0.60, CLOUD, CLOUD_DK, size),
        Glyph::PartlyDay => {
            let mut v = sun(size, 0.34, 0.36, 0.62);
            v.extend(cloud(0.24, 0.40, 0.70, 0.50, CLOUD, CLOUD_DK, size));
            v
        }
        Glyph::PartlyNight => {
            let mut v = vec![ellipse().fill(MOON).at(0.16, 0.14, 0.40, 0.40)];
            v.extend(cloud(0.24, 0.40, 0.70, 0.50, CLOUD, CLOUD_DK, size));
            v
        }
        Glyph::Fog => {
            let mut v = cloud(0.06, 0.10, 0.88, 0.52, CLOUD, CLOUD_DK, size);
            for i in 0..3 {
                let y = 0.72 + i as f64 * 0.11;
                v.push(line((0.16, y), (0.84, y)).stroke(FOGLINE, size * 0.055));
            }
            v
        }
        Glyph::Drizzle => {
            let mut v = cloud(0.06, 0.12, 0.88, 0.54, CLOUD, CLOUD_DK, size);
            v.extend(drops(0.80, 0.92, size));
            v
        }
        Glyph::Rain => {
            let mut v = cloud(0.06, 0.10, 0.88, 0.54, CLOUD, CLOUD_DK, size);
            v.extend(drops(0.78, 0.98, size));
            v
        }
        Glyph::Snow => {
            let mut v = cloud(0.06, 0.10, 0.88, 0.54, CLOUD, CLOUD_DK, size);
            for (i, fx) in [0.30, 0.50, 0.70].into_iter().enumerate() {
                let cy = if i == 1 { 0.90 } else { 0.84 };
                v.push(ellipse().fill(SNOW).at(fx - 0.045, cy - 0.045, 0.09, 0.09));
            }
            v
        }
        Glyph::Thunder => {
            let mut v = cloud(0.06, 0.08, 0.88, 0.54, CLOUD, CLOUD_DK, size);
            v.push(
                polygon([
                    (0.52, 0.66),
                    (0.40, 0.90),
                    (0.50, 0.90),
                    (0.44, 1.02),
                    (0.62, 0.80),
                    (0.52, 0.80),
                    (0.60, 0.66),
                ])
                .fill(BOLT),
            );
            v
        }
    }
}

/// A sun centred at (cxf, cyf) with radius scaled by `scale`: eight rays, then the disc + core.
fn sun(size: f64, cxf: f64, cyf: f64, scale: f64) -> Vec<ShapePiece> {
    let rad = 0.20 * scale;
    let mut v = Vec::with_capacity(10);
    for k in 0..8 {
        let a = k as f64 * std::f64::consts::FRAC_PI_4;
        let (sa, ca) = a.sin_cos();
        let r1 = rad + 0.06 * scale;
        let r2 = rad + 0.15 * scale;
        v.push(
            line(
                (cxf + ca * r1, cyf + sa * r1),
                (cxf + ca * r2, cyf + sa * r2),
            )
            .stroke(SUN, size * 0.045 * scale),
        );
    }
    v.push(
        ellipse()
            .fill(SUN)
            .at(cxf - rad, cyf - rad, rad * 2.0, rad * 2.0),
    );
    v.push(
        ellipse()
            .fill(SUN_CORE)
            .at(cxf - rad * 0.62, cyf - rad * 0.62, rad * 1.24, rad * 1.24),
    );
    v
}

/// A crescent-ish moon filling most of the square (two craters for character).
fn moon() -> Vec<ShapePiece> {
    vec![
        ellipse().fill(MOON).at(0.26, 0.16, 0.52, 0.52),
        ellipse().fill(CLOUD_DK).at(0.40, 0.26, 0.08, 0.08),
        ellipse().fill(CLOUD_DK).at(0.55, 0.44, 0.06, 0.06),
    ]
}

/// A cloud within the fractional box (x, y, w, h): a rounded body plus three overlapping puffs,
/// in two tones for depth. The body's corner radius is 0.18 of the cloud height, in points.
fn cloud(x: f64, y: f64, w: f64, h: f64, col: Color, shadow: Color, size: f64) -> Vec<ShapePiece> {
    let corner = 0.18 * h * size;
    vec![
        // Soft shadow offset a touch down/right.
        rounded_rectangle(corner)
            .fill(shadow)
            .at(x + 0.10 * w, y + 0.60 * h, 0.84 * w, 0.36 * h),
        // Body + puffs.
        rounded_rectangle(corner)
            .fill(col)
            .at(x + 0.08 * w, y + 0.54 * h, 0.84 * w, 0.36 * h),
        ellipse()
            .fill(col)
            .at(x + 0.06 * w, y + 0.36 * h, 0.42 * w, 0.42 * h),
        ellipse()
            .fill(col)
            .at(x + 0.30 * w, y + 0.20 * h, 0.46 * w, 0.46 * h),
        ellipse()
            .fill(col)
            .at(x + 0.52 * w, y + 0.34 * h, 0.40 * w, 0.40 * h),
    ]
}

/// Three slanted rain streaks between fractional y `top`…`bot`.
fn drops(top: f64, bot: f64, size: f64) -> Vec<ShapePiece> {
    [0.34, 0.52, 0.70]
        .into_iter()
        .map(|fx| line((fx, top), (fx - 0.05, bot)).stroke(RAINDROP, size * 0.05))
        .collect()
}
