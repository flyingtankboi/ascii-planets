use crate::stages::shading::ShadeSettings;

/// Settings holds all the graphical settings and choices of the rendering
/// Image characteristic.
/// The ascii technique used : Dithering etc..
#[derive(Debug, Clone)]
pub struct Settings {
    /// Shading -> diffuse model : Lambert, Half Lambert
    pub shade_settings: ShadeSettings,
}
