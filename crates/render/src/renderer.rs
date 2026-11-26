use scene::Scene;

use crate::canvas::Canvas;
use crate::settings::Settings;
use crate::stages::quantize;
use crate::stages::{map, shading, tonemap};

/// Runs the full processing of a frame rendering, from a scene, settings and window size to the canvas
/// 1. Shade
/// 2. Tonemap
/// 3. Color map
/// 4. Quantize
pub fn render(scene: &Scene, settings: &Settings, width: usize, height: usize) -> Canvas {
    let shade = shading::shade(scene, width, height, &settings.shade_settings);
    let tonemap = tonemap::tonemap(shade, &settings.tone_settings);
    let map = map::map_color(tonemap, &settings.map_settings);
    quantize::quantize(map, &settings.quantize_settings)
}
