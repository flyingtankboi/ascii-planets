//! Quantize stage: turns the target to a Cell : a glyph and foreground color
use std::ops::Index;

use crate::canvas::{Canvas, Cell, Color};
use crate::grid::Grid;
use crate::settings::GlyphRampChoice;
use crate::stages::map::Target;

/// Glyph ramps
#[derive(Debug, Clone, PartialEq)]
pub struct GlyphRamp {
    pub ramp: Vec<char>,
}

impl Default for GlyphRamp {
    /// Sets a default for the glyph ramp
    fn default() -> Self {
        GlyphRamp {
            ramp: GlyphRampChoice::Short.chars().to_vec(),
        }
    }
}

impl GlyphRamp {
    /// Gives the length of the ramp
    fn len(&self) -> usize {
        self.ramp.len()
    }
}

impl Index<usize> for GlyphRamp {
    type Output = char;
    /// To use indexing on glyph ramps
    fn index(&self, index: usize) -> &Self::Output {
        &self.ramp[index]
    }
}

/// Settings for the quantize stage.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuantizeSettings {
    pub glyph_ramp: GlyphRamp,
}

/// Quantizes the target
pub(crate) fn quantize(target: Grid<Target>, settings: QuantizeSettings) -> Canvas {
    target.map(|target| {
        let n = settings.glyph_ramp.len() as f32;
        let brightness_index = (target.brightness * n).floor().clamp(0., n - 1.) as usize;
        let symbol = settings.glyph_ramp[brightness_index];
        let fg = Color(
            to_u8_color(target.fg_color.x),
            to_u8_color(target.fg_color.y),
            to_u8_color(target.fg_color.z),
        );
        Cell {
            symbol: Some(symbol),
            fg,
            bg: None,
        }
    })
}

fn to_u8_color(x: f32) -> u8 {
    (x * 255.).round().clamp(0., 255.) as u8
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Vec3;

    fn quantize_one(brightness: f32, ramp: &[char]) -> Cell {
        let target = Target {
            fg_color: Vec3::ZERO,
            brightness,
        };
        let settings = QuantizeSettings {
            glyph_ramp: GlyphRamp {
                ramp: ramp.to_vec(),
            },
        };
        quantize(Grid::new(1, 1, target), settings)[(0, 0)]
    }

    #[test]
    fn test_brightness_to_glyph() {
        let ramp = ['a', 'b', 'c', 'd'];
        assert_eq!(quantize_one(0., &ramp).symbol, Some('a'));
        assert_eq!(quantize_one(0.5, &ramp).symbol, Some('c'));
        assert_eq!(quantize_one(0.99, &ramp).symbol, Some('d'));
        // Full brightness must not index past the ramp
        assert_eq!(quantize_one(1., &ramp).symbol, Some('d'));
    }

    #[test]
    fn test_to_u8_color() {
        assert_eq!(to_u8_color(-1.), 0);
        assert_eq!(to_u8_color(0.5), 128);
        assert_eq!(to_u8_color(2.), 255);
    }
}
