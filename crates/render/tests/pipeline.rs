//! End to end tests of the render pipeline, from a scene to a canvas.

use std::f32::consts::{FRAC_PI_2, PI};

use glam::{Quat, Vec3};
use render::canvas::Canvas;
use render::renderer::render;
use render::settings::{GlyphRampChoice, Settings};
use render::stages::map::{ColorMode, MapSettings};
use render::stages::quantize::QuantizeSettings;
use render::stages::shading::ShadeSettings;
use render::stages::tonemap::{Operator, ToneMapSettings};
use scene::Scene;
use scene::camera::{Camera, CameraType};
use scene::geometry::Pose;
use scene::light::{Light, LightType};
use scene::planet::Planet;

const WIDTH: usize = 20;
const HEIGHT: usize = 10;

fn settings() -> Settings {
    Settings {
        shade_settings: ShadeSettings::default(),
        tone_settings: ToneMapSettings::default(),
        map_settings: MapSettings::default(),
        quantize_settings: QuantizeSettings::default(),
    }
}

fn camera(orientation: Quat) -> Camera {
    Camera::with_pose(
        Pose::new(Vec3::ZERO, orientation),
        CameraType::Perspective {
            fov_radian: FRAC_PI_2,
        },
    )
}

fn light(light_type: LightType, orientation: Quat) -> Light {
    Light {
        pose: Pose::new(Vec3::ZERO, orientation),
        light_type,
        color: Vec3::ONE,
    }
}

/// One planet in front of the camera lit by an ambient light
fn lit_scene(radius: f32) -> Scene {
    Scene {
        planets: vec![Planet::from_geometry(Vec3::new(0., 0., 5.), radius)],
        lights: vec![light(LightType::AmbientLight, Quat::IDENTITY)],
        camera: camera(Quat::IDENTITY),
    }
}

/// Index of the cell's glyph in the default ramp, 0 being the darkest
fn glyph_level(canvas: &Canvas, x: usize, y: usize) -> usize {
    let symbol = canvas[(x, y)].symbol.unwrap();
    GlyphRampChoice::Short
        .chars()
        .iter()
        .position(|&c| c == symbol)
        .unwrap()
}

fn lit_cells(canvas: &Canvas) -> usize {
    canvas
        .iter()
        .filter(|(_, _, cell)| cell.symbol != Some(' '))
        .count()
}

#[test]
fn test_empty_scene_is_dark() {
    let scene = Scene {
        planets: vec![],
        lights: vec![],
        camera: camera(Quat::IDENTITY),
    };
    let canvas = render(&scene, &settings(), WIDTH, HEIGHT);

    assert_eq!((canvas.width(), canvas.height()), (WIDTH, HEIGHT));
    assert_eq!(lit_cells(&canvas), 0);
}

#[test]
fn test_planet_lit_in_center_only() {
    let canvas = render(&lit_scene(1.), &settings(), WIDTH, HEIGHT);

    assert!(glyph_level(&canvas, WIDTH / 2, HEIGHT / 2) > 0);
    assert_eq!(glyph_level(&canvas, 0, 0), 0);
    assert_eq!(glyph_level(&canvas, WIDTH - 1, HEIGHT - 1), 0);
}

#[test]
fn test_camera_facing_away_is_dark() {
    let mut scene = lit_scene(1.);
    scene.camera = camera(Quat::from_rotation_y(PI));
    let canvas = render(&scene, &settings(), WIDTH, HEIGHT);

    assert_eq!(lit_cells(&canvas), 0);
}

#[test]
fn test_bigger_planet_covers_more_cells() {
    let small = render(&lit_scene(1.), &settings(), WIDTH, HEIGHT);
    let big = render(&lit_scene(2.), &settings(), WIDTH, HEIGHT);

    assert!(lit_cells(&big) > lit_cells(&small));
}

#[test]
fn test_side_light_brightens_one_side() {
    // Light shining towards +X lights the -X side, which is the left of the screen
    let mut scene = lit_scene(2.);
    scene.lights = vec![light(
        LightType::DirectionalLight,
        Quat::from_rotation_y(FRAC_PI_2),
    )];
    let canvas = render(&scene, &settings(), WIDTH, HEIGHT);

    let (x, y) = (WIDTH / 2, HEIGHT / 2);
    assert!(glyph_level(&canvas, x - 2, y) > glyph_level(&canvas, x + 2, y));
}

#[test]
fn test_mono_color_mode() {
    let mut settings = settings();
    settings.map_settings.color_mode = ColorMode::Mono {
        color: Vec3::new(1., 0., 0.),
    };
    let canvas = render(&lit_scene(1.), &settings, WIDTH, HEIGHT);

    let fg = canvas[(WIDTH / 2, HEIGHT / 2)].fg;
    assert_eq!((fg.0, fg.1, fg.2), (255, 0, 0));
}

#[test]
fn test_overexposure_does_not_panic() {
    let mut settings = settings();
    settings.tone_settings.exposure_stops = 10.;
    settings.tone_settings.operator = Operator::Clamp { clamp: 1. };
    let canvas = render(&lit_scene(1.), &settings, WIDTH, HEIGHT);

    assert_eq!(canvas[(WIDTH / 2, HEIGHT / 2)].symbol, Some('@'));
}

#[test]
fn test_tiny_canvas_sizes() {
    assert_eq!(render(&lit_scene(1.), &settings(), 0, 0).cells.len(), 0);
    assert_eq!(render(&lit_scene(1.), &settings(), 1, 1).cells.len(), 1);
}
