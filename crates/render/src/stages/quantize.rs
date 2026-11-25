//! Quantize stage: turns the target to a Cell : a glyph and foreground color
use std::ops::Index;

use crate::stages::map::Target;
use crate::grid::Grid;
use crate::canvas::{Canvas, Color, Cell};

/// Classic short ASCII ramp, darkest to lightest.
const SHORT: &[char] = &[' ', '.', ':', '-', '=', '+', '*', '#', '%', '@'];

/// Unicode block shading ramp, darkest to lightest.
const BLOCKS: &[char] = &[' ', '░', '▒', '▓', '█'];

/// Minimal 5-step ramp, very bold.
const MINIMAL: &[char] = &[' ', '.', 'o', 'O', '@'];

/// Rounded, soft-looking ramp.
const ROUND: &[char] = &[' ', '.', ':', 'o', 'O', '0', '@'];

/// Unicode dots of increasing size.
const DOTS: &[char] = &[' ', '·', '•', '●'];

/// Medium ramp, a good middle ground between short and long.
const MEDIUM: &[char] = &[
    ' ', '.', '\'', ':', ';', '-', '~', '=', '+', '*', 'o', 'x', '#', '%', '&', '@',
];

/// Paul Bourke's 70-character ramp, reversed for light-on-dark.
const LONG: &[char] = &[
    ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_',
    '-', '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n',
    'u', 'v', 'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p',
    'd', 'b', 'k', 'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$',
];

/// Which glyph ramp the quantize stage uses, ordered darkest to lightest.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum GlyphRampChoice {
    /// Short classic ramp: ` .:-=+*#%@`.
    #[default]
    Short,
    /// Unicode block shading: ` ░▒▓█`.
    Blocks,
    /// Minimal 5-step ramp: ` .oO@`.
    Minimal,
    /// Rounded, soft-looking ramp: ` .:oO0@`.
    Round,
    /// Unicode dots of increasing size: ` ·•●`.
    Dots,
    /// Medium 16-step ramp: ` .':;-~=+*ox#%&@`.
    Medium,
    /// Paul Bourke's 70-character ramp
    Long,
    /// A user-supplied ramp.
    Custom(GlyphRamp),
}

impl GlyphRampChoice {
    /// The ramp's characters, darkest first.
    pub fn chars(&self) -> &[char] {
        match self {
            Self::Short => SHORT,
            Self::Blocks => BLOCKS,
            Self::Minimal => MINIMAL,
            Self::Round => ROUND,
            Self::Dots => DOTS,
            Self::Medium => MEDIUM,
            Self::Long => LONG,
            Self::Custom(ramp) => &ramp.ramp,
        }
    }
}


/// Glyph ramps 
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GlyphRamp {
    ramp : Vec<char>
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
    fn index(&self, index : usize) -> &Self::Output {
        &self.ramp[index]
    }
}



/// Settings for the quantize stage.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct QuantizeSettings {
    glyph_ramp : GlyphRamp,
}

/// Quantizes the target
pub(crate) fn quantize(target : Grid<Target>, settings : QuantizeSettings) -> Canvas {
    target.map(|target| {

        let brightness_index = (target.brightness * settings.glyph_ramp.len() as f32).floor() as usize;
        let symbol = settings.glyph_ramp[brightness_index];
        let fg = Color(to_u8_color(target.fg_color.x), to_u8_color(target.fg_color.y), to_u8_color(target.fg_color.z));
        Cell{
            symbol : Some(symbol),
            fg,
            bg : None,
        }
    })
}

fn to_u8_color(x : f32) -> u8 {
    (x*255.).round().clamp(0., 255.) as u8
}