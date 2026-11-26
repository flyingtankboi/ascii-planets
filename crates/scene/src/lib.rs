//! Scene structure holding everything the renderer needs to render
use crate::camera::Camera;
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
