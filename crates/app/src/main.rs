//! Terminal based ascii planet renderer
use std::f32::consts::FRAC_PI_3;
use std::time::Duration;

use crossterm::event::{self, KeyCode};
use crossterm::terminal;
use glam::{Quat, Vec3};
use ratatui::{DefaultTerminal, Frame};
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

use crate::scene_widget::SceneWidget;

mod scene_widget;

/// General Result handler.
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Main call to run the ratatui app.
fn main() -> Result<()> {
    ratatui::run(run)
}

/// Run the application loop.
/// Exits when the user presses 'q'.
fn run(terminal: &mut DefaultTerminal) -> Result<()> {
    let scene = demo_complex_scene();
    let settings = demo_settings();
    loop {
        terminal.draw(|frame| render(frame, &scene, &settings))?;
        if should_quit()? {
            break;
        }
    }
    Ok(())
}

/// Render the application
fn render(frame: &mut Frame, scene: &Scene, settings: &Settings) {
    frame.render_widget(SceneWidget { scene, settings }, frame.area());
}

/// Check if the user has pressed 'q' every 250ms.
fn should_quit() -> Result<bool> {
    if event::poll(Duration::from_millis(250))? {
        let q_pressed = event::read()?
            .as_key_press_event()
            .is_some_and(|key| key.code == KeyCode::Char('q'));
        return Ok(q_pressed);
    }
    Ok(false)
}

/// Height / width ratio of one terminal cell, from the window size in pixels.
/// Returns None if the terminal doesn't report its pixel size.
fn terminal_cell_aspect() -> Option<f32> {
    let size = terminal::window_size().ok()?;
    if size.width == 0 || size.height == 0 || size.columns == 0 || size.rows == 0 {
        return None;
    }
    let cell_width = size.width as f32 / size.columns as f32;
    let cell_height = size.height as f32 / size.rows as f32;
    Some(cell_height / cell_width)
}

/// Temporary demo scene: one planet lit from the top left, with a dim ambient light.
/// TODO: replace once scenes can be loaded.
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
/// TODO: replace once scenes can be loaded.
fn demo_complex_scene() -> Scene {
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
            Planet::from_geometry(Vec3::new(-2.5, -1., 6.), 1.),
            // Small moon in front of the giant, on the right
            Planet::from_geometry(Vec3::new(1.8, -1.3, 4.5), 0.4),
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

/// Temporary demo settings.
/// TODO: replace once settings can be loaded.
fn demo_settings() -> Settings {
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
