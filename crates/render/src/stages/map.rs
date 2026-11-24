//! Map stage: turns the linear RGB to sRGB and gives a target for the glyphs :
//! A glyph brightness and foreground color

use glam::Vec3;

use crate::stages::tonemap::ToneFragment;
use crate::grid::Grid;

/// The coloring mode that choses how we map colors to glyph brightness + foreground color
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorMode {
    /// Monochrome color used, the color you give is sRGB like what you see on screen
    Mono{color : Vec3},
    /// The color as is in sRGB
    #[default]
    Full,
    /// The color with the brighness removed in sRGB
    Hue,
}

/// Settings for the color map stage.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct MapSettings {
    /// [`ColorMode`] setting
    pub color_mode: ColorMode,
}

/// Output of the map stage for one cell.
/// It's the target for color and brighness ASCII quantization needs to aim for
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct Target {
    /// fg_color sRGB in [0,1]
    pub fg_color: Vec3,
    /// Brightness value in [0,1]
    pub brightness : f32,
}

/// Maps the tonemap RGB bounded colors to a fg_color and a perceived brightness to achieve
pub fn map_color(tonemap_grid: Grid<ToneFragment>, settings: MapSettings) -> Grid<Target> {
    tonemap_grid.map(|shade| {

        let luminance = shade.color.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let perceived_brightness = to_srgb(luminance);

        let color = match settings.color_mode {
            ColorMode::Mono{color} => color,
            ColorMode::Full => shade.color.map(to_srgb),
            ColorMode::Hue => {
                let max_channel = shade.color.max_element();
                if max_channel == 0. {
                    Vec3::ZERO
                } else {
                    shade.color.map(|x|to_srgb(x / shade.color.max_element()))
                }
            },
        };
        Target {
            fg_color : color,
            brightness : perceived_brightness,
        }
    })
}

fn to_srgb(x : f32) -> f32 {
    if x <= 0.0031308 {
        12.92 * x 
    } else {
        1.055 * x.powf(1./2.4)  - 0.055
    }
}