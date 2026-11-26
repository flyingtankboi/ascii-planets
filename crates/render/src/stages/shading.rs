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

#[cfg(test)]
mod tests {
    use super::*;
    use glam::Quat;
    use scene::camera::{Camera, CameraType};
    use scene::geometry::Pose;
    use scene::planet::Planet;
    use std::f32::consts::{FRAC_PI_2, PI};

    const EPSILON: f32 = 1e-5;

    fn assert_vec3_approx_eq(actual: Vec3, expected: Vec3) {
        assert!(
            (actual - expected).length() < EPSILON,
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    fn light(light_type: LightType, orientation: Quat, color: Vec3) -> Light {
        Light {
            pose: Pose::new(Vec3::ZERO, orientation),
            light_type,
            color,
        }
    }

    /// Hit on the front of a unit planet at z = 5, facing the camera at the origin
    fn front_hit(planet: &Planet) -> Hit<'_> {
        Hit {
            distance: 4.,
            point: Vec3::new(0., 0., 4.),
            normal: Vec3::NEG_Z,
            planet,
        }
    }

    #[test]
    fn test_diffuse_models() {
        assert_eq!(diffuse(1., DiffuseModel::Lambert), 1.);
        assert_eq!(diffuse(-1., DiffuseModel::Lambert), 0.);
        assert_eq!(diffuse(1., DiffuseModel::HalfLambert), 1.);
        assert_eq!(diffuse(0., DiffuseModel::HalfLambert), 0.25);
        assert_eq!(diffuse(-1., DiffuseModel::HalfLambert), 0.);
    }

    #[test]
    fn test_ambient_light() {
        let planet = Planet::from_geometry(Vec3::new(0., 0., 5.), 1.);
        let ambient = light(LightType::AmbientLight, Quat::IDENTITY, Vec3::splat(0.3));
        let color = shade_hit(&front_hit(&planet), &[ambient], &ShadeSettings::default());
        assert_vec3_approx_eq(color, Vec3::splat(0.3));
    }

    #[test]
    fn test_directional_light_facing() {
        let planet = Planet::from_geometry(Vec3::new(0., 0., 5.), 1.);
        let hit = front_hit(&planet);
        let settings = ShadeSettings::default();

        // Shining +Z from behind the camera lights the front fully
        let front = light(LightType::DirectionalLight, Quat::IDENTITY, Vec3::ONE);
        assert_vec3_approx_eq(shade_hit(&hit, &[front], &settings), Vec3::ONE);

        // Shining -Z lights the back, so the front is dark
        let back = light(
            LightType::DirectionalLight,
            Quat::from_rotation_y(PI),
            Vec3::ONE,
        );
        assert_vec3_approx_eq(shade_hit(&hit, &[back], &settings), Vec3::ZERO);

        // Contributions add up
        assert_vec3_approx_eq(shade_hit(&hit, &[front, front], &settings), Vec3::splat(2.));
    }

    #[test]
    fn test_shade_hit_and_miss() {
        let scene = Scene {
            planets: vec![Planet::from_geometry(Vec3::new(0., 0., 5.), 1.)],
            lights: vec![light(LightType::AmbientLight, Quat::IDENTITY, Vec3::ONE)],
            camera: Camera {
                pose: Pose::default(),
                camera_type: CameraType::Perspective {
                    fov_radian: FRAC_PI_2,
                },
            },
        };
        let grid = shade(&scene, 9, 5, &ShadeSettings::default());

        assert_eq!((grid.width, grid.height), (9, 5));
        assert!(grid[(4, 2)].hit);
        assert_eq!(grid[(0, 0)], ShadeFragment::default());
    }
}
