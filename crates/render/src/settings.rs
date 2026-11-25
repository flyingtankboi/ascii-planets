use crate::stages::{
    map::MapSettings,
    quantize::{GlyphRamp, QuantizeSettings},
    shading::ShadeSettings,
    tonemap::ToneMapSettings,
};

/// Settings holds all the graphical settings and choices of the rendering
/// Image characteristic.
/// The ascii technique used : Dithering etc..
#[derive(Debug, Clone)]
pub struct Settings {
    /// Shading -> diffuse model : Lambert, Half Lambert
    pub shade_settings: ShadeSettings,
    pub tone_settings: ToneMapSettings,
    pub map_settings: MapSettings,
    pub quantize_settings: QuantizeSettings,
}

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
    ' ', '.', '\'', '`', '^', '"', ',', ':', ';', 'I', 'l', '!', 'i', '>', '<', '~', '+', '_', '-',
    '?', ']', '[', '}', '{', '1', ')', '(', '|', '\\', '/', 't', 'f', 'j', 'r', 'x', 'n', 'u', 'v',
    'c', 'z', 'X', 'Y', 'U', 'J', 'C', 'L', 'Q', '0', 'O', 'Z', 'm', 'w', 'q', 'p', 'd', 'b', 'k',
    'h', 'a', 'o', '*', '#', 'M', 'W', '&', '8', '%', 'B', '@', '$',
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
