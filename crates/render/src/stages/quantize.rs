//! Quantize stage: turns the target to a Cell : a glyph and foreground color
use std::ops::Index;

use crate::canvas::{Canvas, Cell, Color};
use crate::grid::Grid;
use crate::stages::map::Target;

/// Glyph ramps
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlyphRamp {
    pub ramp: Vec<char>,
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
    glyph_ramp: GlyphRamp,
}

/// Quantizes the target
pub(crate) fn quantize(target: Grid<Target>, settings: QuantizeSettings) -> Canvas {
    target.map(|target| {
        let brightness_index =
            (target.brightness * settings.glyph_ramp.len() as f32).floor() as usize;
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
