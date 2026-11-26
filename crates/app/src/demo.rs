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
    Simple,
    Color,
}

impl DemoOption {
    /// Gives the corresponding demo scene and settings
    pub fn fetch(&self) -> (Scene, Settings) {
        let scene = match self {
            DemoOption::Simple => demo_scene(),
            DemoOption::Color => demo_color_scene(),
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
pub fn demo_scene() -> Scene {
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
        camera: Camera {
            pose: Pose::default(),
            camera_type: CameraType::Perspective {
                fov_radian: FRAC_PI_3,
            },
        },
    }
}

/// Temporary complex demo scene: a gas giant with two moons, lit by a warm sun from the
/// front left, a cyan rim light from behind and a magenta glow from below.
pub fn demo_color_scene() -> Scene {
    // Directional light travelling along `direction`
    let directional = |direction: Vec3, color: Vec3| Light {
        pose: Pose::new(
            Vec3::ZERO,
            Quat::from_rotation_arc(Vec3::Z, direction.normalize()),
        ),
        light_type: LightType::DirectionalLight,
        color,
    };

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
            directional(Vec3::new(1., -0.4, 0.6), Vec3::new(1.6, 0.9, 0.4)),
            directional(Vec3::new(-1., 0.1, -0.3), Vec3::new(0.2, 0.9, 1.8)),
            directional(Vec3::new(0., 1., 0.2), Vec3::new(0.8, 0.1, 0.6)),
            Light {
                pose: Pose::default(),
                light_type: LightType::AmbientLight,
                color: Vec3::new(0.02, 0.01, 0.04),
            },
        ],
        camera: Camera {
            pose: Pose::default(),
            camera_type: CameraType::Perspective {
                fov_radian: FRAC_PI_3,
            },
        },
    }
}

/// Demo settings for all the demos
pub fn demo_settings() -> Settings {
    Settings {
        shade_settings: ShadeSettings {
            diffuse: DiffuseModel::Lambert,
            cell_aspect: terminal_cell_aspect().unwrap_or(ShadeSettings::default().cell_aspect),
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
