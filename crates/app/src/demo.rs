//! Demo scenes you can load with command line --demo

use clap::ValueEnum;
use crossterm::terminal;
use glam::{Quat, Vec3};
use std::f32::consts::FRAC_PI_3;

use render::settings::{GlyphRampChoice, Settings};
use render::stages::map::MapSettings;
use render::stages::quantize::{GlyphRamp, QuantizeSettings};
use render::stages::shading::{DiffuseModel, ShadeSettings};
use render::stages::tonemap::ToneMapSettings;
use scene::Scene;
use scene::camera::{Camera, CameraType};
use scene::geometry::Pose;
use scene::light::{Light, LightType};
use scene::planet::Planet;

/// Enum to store the different demos
#[derive(Clone, ValueEnum)]
pub enum DemoOption {
    /// Simple planet
    Simple,
    /// Three planets with colored lights
    Color,
    /// A moon casting its shadow on a planet
    Eclipse,
    /// Colored point lights and a spot light, with a moon in the spot light's beam
    Lights,
}

impl DemoOption {
    /// Gives the corresponding demo scene and settings
    pub fn build(&self) -> (Scene, Settings) {
        let scene = match self {
            DemoOption::Simple => demo_scene(),
            DemoOption::Color => demo_color_scene(),
            DemoOption::Eclipse => demo_eclipse_scene(),
            DemoOption::Lights => demo_lights_scene(),
        };
        (scene, demo_settings())
    }
}

/// Height / width ratio of one terminal cell, from the window size in pixels.
/// Returns None if the terminal doesn't report its pixel size.
pub fn terminal_cell_aspect() -> Option<f32> {
    let size = terminal::window_size().ok()?;
    if size.width == 0 || size.height == 0 || size.columns == 0 || size.rows == 0 {
        return None;
    }
    let cell_width = size.width as f32 / size.columns as f32;
    let cell_height = size.height as f32 / size.rows as f32;
    Some(cell_height / cell_width)
}

/// Simple demo scene: one planet lit from the top left, with a dim ambient light.
fn demo_scene() -> Scene {
    let sun_direction = Vec3::new(1., -0.5, 1.).normalize();
    Scene {
        planets: vec![Planet::from_geometry(Vec3::new(0., 0., 5.), 2.)],
        lights: vec![
            Light {
                pose: Pose::new(Vec3::ZERO, Quat::from_rotation_arc(Vec3::Z, sun_direction)),
                light_type: LightType::DirectionalLight,
                color: Vec3::new(1., 1., 0.),
            },
            Light {
                pose: Pose::default(),
                light_type: LightType::AmbientLight,
                color: Vec3::new(0., 0.1, 0.1),
            },
        ],
        camera: Camera::new(CameraType::Perspective {
            fov_radian: FRAC_PI_3,
        }),
    }
}

/// Temporary complex demo scene: three planets, lit by a warm sun from the
/// front left, a cyan rim light from behind and a magenta glow from below.
fn demo_color_scene() -> Scene {
    Scene {
        planets: vec![
            // Gas giant in the back
            Planet::from_geometry(Vec3::new(1.5, 0.5, 10.), 3.5),
            // Moon in front, on the left
            Planet::from_geometry(Vec3::new(-2.5, -1., 6.), 2.),
            // Small moon in front of the giant, on the right
            Planet::from_geometry(Vec3::new(1.8, -1.3, 4.5), 1.1),
        ],
        lights: vec![
            directional_light(Vec3::new(1., -0.4, 0.6), Vec3::new(1.6, 0.9, 0.4)),
            directional_light(Vec3::new(-1., 0.1, -0.3), Vec3::new(0.2, 0.9, 1.8)),
            directional_light(Vec3::new(0., 1., 0.2), Vec3::new(0.8, 0.1, 0.6)),
            ambient_light(Vec3::new(0.02, 0.01, 0.04)),
        ],
        camera: Camera::new(CameraType::Perspective {
            fov_radian: FRAC_PI_3,
        }),
    }
}

/// Eclipse demo: a sun from the left, a moon between it and the planet casting a round
/// shadow on the lit side.
fn demo_eclipse_scene() -> Scene {
    let sun_direction = Vec3::new(1., -0.2, 0.4).normalize();
    // Point on the lit side of the planet where the shadow lands, moon placed towards the sun
    let shadow_center = 2. * Vec3::new(-0.6, 0.2, -0.75).normalize();
    let moon_position = shadow_center - 1.3 * sun_direction;
    Scene {
        planets: vec![
            Planet::from_geometry(Vec3::ZERO, 2.),
            Planet::from_geometry(moon_position, 0.5),
        ],
        lights: vec![
            directional_light(sun_direction, Vec3::new(1.4, 1.2, 1.)),
            ambient_light(Vec3::new(0.01, 0.01, 0.02)),
        ],
        // Turned towards the sun so the moon is in view, the orbit camera keeps this orientation
        camera: Camera::with_pose(
            Pose::new(Vec3::ZERO, Quat::from_rotation_y(0.6)),
            CameraType::Perspective {
                fov_radian: FRAC_PI_3,
            },
        ),
    }
}

/// Lights demo: a red and a blue point light close to the surface, and a warm spot light
/// drawing a disc on the planet, with a small moon in its beam casting a shadow inside it.
fn demo_lights_scene() -> Scene {
    let spot_position = Vec3::new(0.6, 1.8, -4.);
    let spot_target = Vec3::new(0.2, 0.3, -1.95);
    Scene {
        planets: vec![
            Planet::from_geometry(Vec3::ZERO, 2.),
            // Closer to the surface than to the spot light, so its shadow fits inside the disc
            Planet::from_geometry(spot_position.lerp(spot_target, 0.75), 0.15),
        ],
        lights: vec![
            point_light(Vec3::new(-2.6, 1., -1.6), Vec3::new(6., 0.5, 0.3)),
            point_light(Vec3::new(2.4, -1.4, -1.8), Vec3::new(0.3, 0.6, 3.5)),
            spot_light(spot_position, spot_target, 0.2, Vec3::new(9., 8., 6.)),
            ambient_light(Vec3::new(0.01, 0.01, 0.02)),
        ],
        // Seen from the right so the moon doesn't hide its own shadow
        camera: Camera::with_pose(
            Pose::new(Vec3::ZERO, Quat::from_rotation_y(-0.5)),
            CameraType::Perspective {
                fov_radian: FRAC_PI_3,
            },
        ),
    }
}

/// Directional light travelling along `direction`
fn directional_light(direction: Vec3, color: Vec3) -> Light {
    Light {
        pose: Pose::new(
            Vec3::ZERO,
            Quat::from_rotation_arc(Vec3::Z, direction.normalize()),
        ),
        light_type: LightType::DirectionalLight,
        color,
    }
}

/// Point light at `position`, its color is divided by the distance squared
fn point_light(position: Vec3, color: Vec3) -> Light {
    Light {
        pose: Pose::new(position, Quat::IDENTITY),
        light_type: LightType::PointLight,
        color,
    }
}

/// Spot light at `position` aimed at `target`, with a cone of half-angle `angle` in radian
fn spot_light(position: Vec3, target: Vec3, angle: f32, color: Vec3) -> Light {
    let direction = (target - position).normalize();
    Light {
        pose: Pose::new(position, Quat::from_rotation_arc(Vec3::Z, direction)),
        light_type: LightType::SpotLight { angle },
        color,
    }
}

/// Ambient light coming from everywhere
fn ambient_light(color: Vec3) -> Light {
    Light {
        pose: Pose::default(),
        light_type: LightType::AmbientLight,
        color,
    }
}

/// Demo settings for all the demos
fn demo_settings() -> Settings {
    Settings {
        shade_settings: ShadeSettings {
            diffuse: DiffuseModel::Lambert,
            cell_aspect: terminal_cell_aspect().unwrap_or(ShadeSettings::default().cell_aspect),
            samples_x: 2,
            samples_y: 4,
        },
        tone_settings: ToneMapSettings::default(),
        map_settings: MapSettings::default(),
        quantize_settings: QuantizeSettings {
            glyph_ramp: GlyphRamp {
                ramp: GlyphRampChoice::Medium.chars().to_vec(),
            },
        },
    }
}
