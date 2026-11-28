//! Shading stage: turns a scene into a grid of linear-radiance fragments.
//!
//! TODO: spot lights, city lights, clouds.

use glam::Vec3;
use scene::{
    Scene,
    geometry::HasPose,
    light::{Light, LightType},
    planet::Planet,
};
use std::iter::Sum;
use std::ops::{Add, Div};

use crate::trace::{
    intersect::{Hit, intersect_planets, intersect_with_one},
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
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ShadeSettings {
    /// Diffuse lighting model used for directional and point lights.
    pub diffuse: DiffuseModel,
    /// Height / width ratio of one cell, terminal cells are usually around 2:1
    pub cell_aspect: f32,
    /// Supersampling: number of samples per cell horizontally, 1 samples the center only
    pub samples_x: usize,
    /// Supersampling: number of samples per cell vertically, 1 samples the center only.
    /// Using twice `samples_x` spreads samples evenly over a 2:1 cell
    pub samples_y: usize,
}

impl Default for ShadeSettings {
    fn default() -> Self {
        ShadeSettings {
            diffuse: DiffuseModel::default(),
            cell_aspect: 2.,
            samples_x: 1,
            samples_y: 1,
        }
    }
}

/// Output of the shading stage for one cell.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub(crate) struct ShadeFragment {
    /// How much of the cell is hitting a planet
    pub hit: f32,
    /// Linear radiance, unbounded.
    pub color: Vec3,
}

impl Add for ShadeFragment {
    type Output = ShadeFragment;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            hit: self.hit + rhs.hit,
            color: self.color + rhs.color,
        }
    }
}

impl Sum for ShadeFragment {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(ShadeFragment::default(), Add::add)
    }
}

impl Div<f32> for ShadeFragment {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Self {
            hit: self.hit / rhs,
            color: self.color / rhs,
        }
    }
}

/// Shades every cell of a `width` × `height` grid, row by row.
/// Each cell is the mean of its samples, in linear radiance so edges blend correctly.
pub(crate) fn shade(
    scene: &Scene,
    width: usize,
    height: usize,
    settings: &ShadeSettings,
) -> Grid<ShadeFragment> {
    let offsets = sample_offsets(settings);
    let sample_count = offsets.len() as f32;
    Grid::new_with(width, height, |x, y| {
        let total: ShadeFragment = offsets
            .iter()
            .map(|(offset_x, offset_y)| {
                let u = (x as f32 + offset_x) / width as f32;
                let v = (y as f32 + offset_y) / height as f32;
                let ray = Ray::from_camera_at_uv(
                    &scene.camera,
                    u,
                    v,
                    width,
                    height,
                    settings.cell_aspect,
                );
                shade_ray(&ray, scene, settings)
            })
            .sum();
        total / sample_count
    })
}

/// Sample positions inside a cell, as a fixed regular grid so edges don't shimmer between frames.
/// Offsets are in `[0, 1]` from the cell's top left corner.
fn sample_offsets(settings: &ShadeSettings) -> Vec<(f32, f32)> {
    let (samples_x, samples_y) = (settings.samples_x.max(1), settings.samples_y.max(1));
    (0..samples_y)
        .flat_map(|j| {
            (0..samples_x).map(move |i| {
                (
                    (i as f32 + 0.5) / samples_x as f32,
                    (j as f32 + 0.5) / samples_y as f32,
                )
            })
        })
        .collect()
}

/// Shades a single ray: full coverage and its light if it hits a planet, nothing otherwise.
fn shade_ray(ray: &Ray, scene: &Scene, settings: &ShadeSettings) -> ShadeFragment {
    match intersect_planets(ray, &scene.planets) {
        Some(hit) => ShadeFragment {
            hit: 1.,
            color: shade_hit(&hit, scene, settings),
        },
        None => ShadeFragment::default(),
    }
}

/// Sums the contribution of every light at one hit.
fn shade_hit(hit: &Hit, scene: &Scene, settings: &ShadeSettings) -> Vec3 {
    scene
        .lights
        .iter()
        .map(|light| light_level(light, hit, scene, settings))
        .sum()
}

/// Light level of one light at one hit, with occlusion from the other planets.
fn light_level(light: &Light, hit: &Hit, scene: &Scene, settings: &ShadeSettings) -> Vec3 {
    let albedo = hit.planet.albedo(hit.point);
    let light_reaching = match light.light_type {
        LightType::AmbientLight => light.color,
        LightType::DirectionalLight => directional_light(light, hit, &scene.planets, settings),
        LightType::PointLight => point_light(light, hit, &scene.planets, settings),
        LightType::SpotLight { angle } => spot_light(light, angle, hit, &scene.planets, settings),
    };
    albedo * light_reaching
}

/// Light reaching a hit from a directional light, all rays are parallel to its forward.
fn directional_light(
    light: &Light,
    hit: &Hit,
    planets: &[Planet],
    settings: &ShadeSettings,
) -> Vec3 {
    let to_light = -light.forward();
    let diffuse = diffuse(hit.normal.dot(to_light), settings.diffuse);
    // No need to trace a shadow ray if the surface faces away anyway
    if diffuse == 0. || is_occluded(hit, planets, &Ray::new(hit.point, to_light), f32::MAX) {
        return Vec3::ZERO;
    }
    light.color * diffuse
}

/// Light reaching a hit from a point light, rays come from its position so its orientation is unused.
fn point_light(light: &Light, hit: &Hit, planets: &[Planet], settings: &ShadeSettings) -> Vec3 {
    let to_light = light.position() - hit.point;
    let distance = to_light.length();
    let to_light = to_light / distance;
    let diffuse = diffuse(hit.normal.dot(to_light), settings.diffuse);
    // Only planets between the hit and the light occlude, hence `t_max = distance`
    if diffuse == 0. || is_occluded(hit, planets, &Ray::new(hit.point, to_light), distance) {
        return Vec3::ZERO;
    }
    // Inverse square fall off
    light.color * diffuse / distance.powi(2)
}

/// Light reaching a hit from a spot light: a point light only lighting inside its cone,
/// which points along its forward with a half-angle of `angle`.
fn spot_light(
    light: &Light,
    angle: f32,
    hit: &Hit,
    planets: &[Planet],
    settings: &ShadeSettings,
) -> Vec3 {
    let from_light = (hit.point - light.position()).normalize();
    if light.forward().dot(from_light) < angle.cos() {
        return Vec3::ZERO;
    }
    point_light(light, hit, planets, settings)
}

/// Whether another planet blocks the shadow ray before `t_max`.
/// The hit planet is skipped: a sphere can't shadow itself, and it would cut the Half Lambert wrap.
fn is_occluded(hit: &Hit, planets: &[Planet], shadow_ray: &Ray, t_max: f32) -> bool {
    planets
        .iter()
        // Compares addresses: `hit.planet` points into `planets`, so this is the same planet
        .filter(|planet| !std::ptr::eq(*planet, hit.planet))
        .any(|planet| intersect_with_one(shadow_ray, planet, 0.001, t_max).is_some())
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

    fn light(light_type: LightType, pose: Pose, color: Vec3) -> Light {
        Light {
            pose,
            light_type,
            color,
        }
    }

    /// Scene with a unit planet at z = 5 first, then the extra planets
    fn scene(lights: Vec<Light>, extra_planets: Vec<Planet>) -> Scene {
        let mut planets = vec![Planet::from_geometry(Vec3::new(0., 0., 5.), 1.)];
        planets.extend(extra_planets);
        Scene {
            planets,
            lights,
            camera: Camera::new(CameraType::Perspective {
                fov_radian: FRAC_PI_2,
            }),
        }
    }

    /// Hit on the front of the first planet, facing the camera at the origin
    fn front_hit(scene: &Scene) -> Hit<'_> {
        Hit {
            distance: 4.,
            point: Vec3::new(0., 0., 4.),
            normal: Vec3::NEG_Z,
            planet: &scene.planets[0],
        }
    }

    /// Small planet between the camera and the front of the first planet
    fn blocker() -> Planet {
        Planet::from_geometry(Vec3::new(0., 0., 2.), 0.5)
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
    fn test_ambient_light_is_never_occluded() {
        let ambient = light(LightType::AmbientLight, Pose::default(), Vec3::splat(0.3));
        let scene = scene(vec![ambient], vec![blocker()]);
        let color = shade_hit(&front_hit(&scene), &scene, &ShadeSettings::default());
        assert_vec3_approx_eq(color, Vec3::splat(0.3));
    }

    #[test]
    fn test_directional_light_facing() {
        let settings = ShadeSettings::default();

        // Shining +Z from behind the camera lights the front fully
        let front = light(LightType::DirectionalLight, Pose::default(), Vec3::ONE);
        let lit = scene(vec![front, front], vec![]);
        // Contributions add up
        assert_vec3_approx_eq(
            shade_hit(&front_hit(&lit), &lit, &settings),
            Vec3::splat(2.),
        );

        // Shining -Z lights the back, so the front is dark
        let back_pose = Pose::new(Vec3::ZERO, Quat::from_rotation_y(PI));
        let back = light(LightType::DirectionalLight, back_pose, Vec3::ONE);
        let dark = scene(vec![back], vec![]);
        assert_vec3_approx_eq(shade_hit(&front_hit(&dark), &dark, &settings), Vec3::ZERO);
    }

    #[test]
    fn test_directional_light_occluded() {
        let front = light(LightType::DirectionalLight, Pose::default(), Vec3::ONE);
        let scene = scene(vec![front], vec![blocker()]);
        let color = shade_hit(&front_hit(&scene), &scene, &ShadeSettings::default());
        assert_vec3_approx_eq(color, Vec3::ZERO);
    }

    #[test]
    fn test_point_light() {
        let settings = ShadeSettings::default();
        // Facing away (-Z) to check the orientation of a point light is ignored
        let pose = Pose::new(Vec3::new(0., 0., 1.), Quat::from_rotation_y(PI));
        let point = light(LightType::PointLight, pose, Vec3::ONE);

        // 3 away from the hit, so inverse square gives 1/9
        let lit = scene(vec![point], vec![]);
        let color = shade_hit(&front_hit(&lit), &lit, &settings);
        assert_vec3_approx_eq(color, Vec3::splat(1. / 9.));

        // A planet between the hit and the light occludes it
        let occluded = scene(vec![point], vec![blocker()]);
        let color = shade_hit(&front_hit(&occluded), &occluded, &settings);
        assert_vec3_approx_eq(color, Vec3::ZERO);

        // A planet behind the light doesn't
        let behind = Planet::from_geometry(Vec3::new(0., 0., -3.), 0.5);
        let not_occluded = scene(vec![point], vec![behind]);
        let color = shade_hit(&front_hit(&not_occluded), &not_occluded, &settings);
        assert_vec3_approx_eq(color, Vec3::splat(1. / 9.));
    }

    #[test]
    fn test_spot_light_cone() {
        let settings = ShadeSettings::default();
        let spot = |orientation| {
            let pose = Pose::new(Vec3::new(0., 0., 1.), orientation);
            light(LightType::SpotLight { angle: 0.1 }, pose, Vec3::ONE)
        };

        // Pointing straight at the hit: lit like a point light
        let inside = scene(vec![spot(Quat::IDENTITY)], vec![]);
        let color = shade_hit(&front_hit(&inside), &inside, &settings);
        assert_vec3_approx_eq(color, Vec3::splat(1. / 9.));

        // Turned 0.2 away, past the 0.1 half-angle: dark
        let outside = scene(vec![spot(Quat::from_rotation_y(0.2))], vec![]);
        let color = shade_hit(&front_hit(&outside), &outside, &settings);
        assert_vec3_approx_eq(color, Vec3::ZERO);
    }

    #[test]
    fn test_shade_hit_and_miss() {
        let ambient = light(LightType::AmbientLight, Pose::default(), Vec3::ONE);
        let grid = shade(
            &scene(vec![ambient], vec![]),
            9,
            5,
            &ShadeSettings::default(),
        );

        assert_eq!((grid.width, grid.height), (9, 5));
        assert_eq!(grid[(4, 2)].hit, 1.);
        assert_eq!(grid[(0, 0)], ShadeFragment::default());
    }

    #[test]
    fn test_supersampling_gives_partial_coverage_on_edges() {
        let ambient = light(LightType::AmbientLight, Pose::default(), Vec3::ONE);
        let scene = scene(vec![ambient], vec![]);
        let partial_cells = |settings: &ShadeSettings| {
            let grid = shade(&scene, 20, 10, settings);
            grid.iter()
                .filter(|(_, _, fragment)| fragment.hit > 0. && fragment.hit < 1.)
                .count()
        };

        // One sample per cell is either a full hit or a full miss
        assert_eq!(partial_cells(&ShadeSettings::default()), 0);

        let supersampled = ShadeSettings {
            samples_x: 2,
            samples_y: 4,
            ..ShadeSettings::default()
        };
        assert!(partial_cells(&supersampled) > 0);
    }

    #[test]
    fn test_sample_offsets_grid() {
        let settings = ShadeSettings {
            samples_x: 2,
            samples_y: 2,
            ..ShadeSettings::default()
        };
        assert_eq!(
            sample_offsets(&settings),
            vec![(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
        );
        assert_eq!(sample_offsets(&ShadeSettings::default()), vec![(0.5, 0.5)]);
    }
}
