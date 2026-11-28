//! Scene structure holding everything the renderer needs to render
use crate::camera::{Camera, OrbitDirection};
use crate::light::Light;
use crate::planet::Planet;

pub mod camera;
pub mod geometry;
pub mod light;
pub mod planet;

/// Represents a Scene as a set of planets, lights and one camera
#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    /// Vec of [`Planet`]
    pub planets: Vec<Planet>,
    /// Vec of [`Light`]
    pub lights: Vec<Light>,
    /// Singular [`Camera`] per scene
    pub camera: Camera,
}

impl Scene {
    /// Steps the movement for time dt in millisecond
    pub fn step(&mut self, dt: f32, direction: &OrbitDirection) {
        for planet in &mut self.planets {
            planet.step(dt);
        }
        if let Some(target) = self
            .planets
            .get(self.camera.target_id)
            .or_else(|| self.planets.first())
        {
            self.camera.step(target, direction, dt);
        }
    }

    pub fn next_camera_target(&mut self) {
        let next_id = (self.camera.target_id + 1) % self.planets.len();
        let next_target = &self.planets[next_id];
        self.camera.set_target(next_target, next_id);
    }
}
