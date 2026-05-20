use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Self = Self { r: 0, g: 0, b: 0 };
    pub const WHITE: Self = Self { r: 255, g: 255, b: 255 };
    pub const RED:   Self = Self { r: 255, g: 0, b: 0 };
    pub const GREEN: Self = Self { r: 0, g: 255, b: 0 };
    pub const BLUE:  Self = Self { r: 0, g: 0, b: 255 };

    pub fn new(r: u8, g: u8, b: u8) -> Self { Self { r, g, b } }

    /// HSV → RGB（h 0-360、s/v 0-1）
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Self {
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r1, g1, b1) = match h as u32 {
            0..=59    => (c, x, 0.0),
            60..=119  => (x, c, 0.0),
            120..=179 => (0.0, c, x),
            180..=239 => (0.0, x, c),
            240..=299 => (x, 0.0, c),
            _         => (c, 0.0, x),
        };
        Self {
            r: ((r1 + m) * 255.0).round() as u8,
            g: ((g1 + m) * 255.0).round() as u8,
            b: ((b1 + m) * 255.0).round() as u8,
        }
    }

    pub fn lerp(self, other: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: lerp_u8(self.r, other.r, t),
            g: lerp_u8(self.g, other.g, t),
            b: lerp_u8(self.b, other.b, t),
        }
    }
}

fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).round().clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsv_round_trip_primary() {
        assert_eq!(Color::from_hsv(0.0, 1.0, 1.0), Color::RED);
        assert_eq!(Color::from_hsv(120.0, 1.0, 1.0), Color::GREEN);
        assert_eq!(Color::from_hsv(240.0, 1.0, 1.0), Color::BLUE);
    }

    #[test]
    fn lerp_endpoints() {
        let mid = Color::BLACK.lerp(Color::WHITE, 0.5);
        assert!((mid.r as i16 - 128).abs() <= 1);
    }
}
