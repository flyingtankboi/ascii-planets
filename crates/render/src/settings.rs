use crate::stages::{map::MapSettings, quantize::QuantizeSettings, shading::ShadeSettings, tonemap::ToneMapSettings};

/// Settings holds all the graphical settings and choices of the rendering
/// Image characteristic.
/// The ascii technique used : Dithering etc..
#[derive(Debug, Clone)]
pub struct Settings {
    /// Shading -> diffuse model : Lambert, Half Lambert
    pub shade_settings: ShadeSettings,
    pub tone_settings : ToneMapSettings,
    pub map_settings : MapSettings,
    pub quantize_settings : QuantizeSettings, 
}

/// Glyph ramp default settings 
#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct GlyphRampDefault {

}