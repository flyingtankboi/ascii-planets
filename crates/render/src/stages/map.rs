//! Map stage: turns the linear RGB to sRGB and gives a target for the glyphs :
//! A glyph brightness and foreground color

use glam::Vec3;

use crate::grid::Grid;
use crate::stages::tonemap::ToneFragment;

/// The coloring mode that choses how we map colors to glyph brightness + foreground color
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ColorMode {
    /// Monochrome color used, the color you give is sRGB like what you see on screen
    Mono { color: Vec3 },
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
    pub(crate) fg_color: Vec3,
    /// Brightness value in [0,1]
    pub(crate) brightness: f32,
}

/// Maps the tonemap RGB bounded colors to a fg_color and a perceived brightness to achieve
pub(crate) fn map_color(tonemap_grid: Grid<ToneFragment>, settings: &MapSettings) -> Grid<Target> {
    tonemap_grid.map(|shade| {
        let luminance = shade.color.dot(Vec3::new(0.2126, 0.7152, 0.0722));
        let perceived_brightness = to_srgb(luminance);

        let color = match settings.color_mode {
            ColorMode::Mono { color } => color,
            ColorMode::Full => shade.color.map(to_srgb),
            ColorMode::Hue => {
                let max_channel = shade.color.max_element();
                if max_channel == 0. {
                    Vec3::ZERO
                } else {
                    shade.color.map(|x| to_srgb(x / shade.color.max_element()))
                }
            }
        };
        Target {
            fg_color: color,
            brightness: perceived_brightness,
        }
    })
}

fn to_srgb(x: f32) -> f32 {
    if x <= 0.0031308 {
        12.92 * x
    } else {
        1.055 * x.powf(1. / 2.4) - 0.055
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_vec3_approx_eq(actual: Vec3, expected: Vec3) {
        assert!(
            (actual - expected).length() < EPSILON,
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    fn map_one(color: Vec3, color_mode: ColorMode) -> Target {
        let grid = Grid::new(1, 1, ToneFragment { color });
        map_color(grid, &MapSettings { color_mode })[(0, 0)]
    }

    #[test]
    fn test_to_srgb() {
        assert_eq!(to_srgb(0.), 0.);
        assert!((to_srgb(1.) - 1.).abs() < EPSILON);
        // Both branches meet at the threshold
        let threshold = 0.0031308;
        assert!((to_srgb(threshold) - to_srgb(threshold + 1e-7)).abs() < 1e-4);
    }

    #[test]
    fn test_full_mode() {
        let white = map_one(Vec3::ONE, ColorMode::Full);
        assert_vec3_approx_eq(white.fg_color, Vec3::ONE);
        assert!((white.brightness - 1.).abs() < EPSILON);
        assert_eq!(map_one(Vec3::ZERO, ColorMode::Full).brightness, 0.);
    }

    #[test]
    fn test_mono_mode_ignores_input() {
        let color = Vec3::new(0.2, 0.4, 0.6);
        let target = map_one(Vec3::new(1., 0., 0.), ColorMode::Mono { color });
        assert_eq!(target.fg_color, color);
    }

    #[test]
    fn test_hue_mode_removes_brightness() {
        let dim = map_one(Vec3::new(0.1, 0.05, 0.), ColorMode::Hue);
        let bright = map_one(Vec3::new(0.8, 0.4, 0.), ColorMode::Hue);
        assert_vec3_approx_eq(dim.fg_color, bright.fg_color);
        assert_eq!(map_one(Vec3::ZERO, ColorMode::Hue).fg_color, Vec3::ZERO);
    }

    #[test]
    fn test_green_brighter_than_blue() {
        let green = map_one(Vec3::Y, ColorMode::Full);
        let blue = map_one(Vec3::Z, ColorMode::Full);
        assert!(green.brightness > blue.brightness);
    }
}
