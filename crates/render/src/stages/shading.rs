//! Shading stage: turns a scene into a grid of linear-radiance fragments.
//!
//! TODO: shadows, city lights, clouds.

use glam::Vec3;
use scene::{
    Scene,
    geometry::HasPose,
    light::{Light, LightType},
};

use crate::trace::{
    intersect::{Hit, intersect_planets},
    ray::Ray,
};

use crate::grid::Grid;

/// How a surface responds to the angle between its normal and the light.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DiffuseModel {
    /// Physically based: brightness follows `max(N·L, 0)`.
    #[default]
    Lambert,
    /// Stylised: light wraps past the terminator, `(N·L * 0.5 + 0.5)²`.
    HalfLambert,
}

/// Settings for the shading stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ShadeSettings {
    /// Diffuse lighting model used for directional lights.
    pub diffuse: DiffuseModel,
}

/// Output of the shading stage for one cell.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct ShadeFragment {
    /// Whether the cell's ray hit a planet.
    pub hit: bool,
    /// Linear radiance, unbounded.
    pub color: Vec3,
}

/// Shades every cell of a `width` × `height` grid, row by row.
pub(crate) fn shade(
    scene: &Scene,
    width: usize,
    height: usize,
    settings: &ShadeSettings,
) -> Grid<ShadeFragment> {
    Ray::from_camera(&scene.camera, width, height).map(|ray| {
        match intersect_planets(ray, &scene.planets) {
            Some(hit) => ShadeFragment {
                hit: true,
                color: shade_hit(&hit, &scene.lights, settings),
            },
            None => ShadeFragment::default(),
        }
    })
}

/// Sums the contribution of every light at one hit.
fn shade_hit(hit: &Hit, lights: &[Light], settings: &ShadeSettings) -> Vec3 {
    lights
        .iter()
        .map(|light| light_level_without_occlusion(light, hit, settings))
        .sum()
}

/// Light level of one light at one hit, ignoring any scene occlusion.
fn light_level_without_occlusion(light: &Light, hit: &Hit, settings: &ShadeSettings) -> Vec3 {
    let albedo = hit.planet.albedo(hit.point);
    match light.light_type {
        LightType::AmbientLight => albedo * light.color,
        LightType::DirectionalLight => {
            let n_dot_l = hit.normal.dot(-light.forward());
            albedo * light.color * diffuse(n_dot_l, settings.diffuse)
        }
        LightType::PointLight | LightType::SpotLight { .. } => todo!("point and spot lights"),
    }
}

/// Diffuse factor in `[0, 1]` for a given `N·L`.
fn diffuse(n_dot_l: f32, model: DiffuseModel) -> f32 {
    match model {
        DiffuseModel::Lambert => n_dot_l.max(0.0),
        DiffuseModel::HalfLambert => (n_dot_l * 0.5 + 0.5).powi(2),
    }
}
