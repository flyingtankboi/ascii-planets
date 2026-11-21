
/// Settings holds all the graphical settings and choices of the rendering
/// Image characteristic.
/// The ascii technique used : Dithering etc..
#[derive(Debug, Clone, Copy)]
pub struct Settings {
    /// Width of the output image
    pub width : usize,
    /// Height of the output image
    pub height : usize,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { width: 600, height: 600 }
    }
}