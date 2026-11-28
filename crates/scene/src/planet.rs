use glam::{Quat, Vec3};

use crate::geometry::{HasPose, Pose, Sphere};

// Represents the planets in the scene
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planet {
    /// Pose in 3D space.
    /// see [`Pose`]
    pub pose: Pose,
    /// The simple geometric form used
    pub geometry: Sphere,
    /// Spin speed along its local Y axis in radian per second
    pub spin_speed: f32,
}

impl HasPose for Planet {
    fn pose(&self) -> &Pose {
        &self.pose
    }

    fn pose_mut(&mut self) -> &mut Pose {
        &mut self.pose
    }
}

impl Planet {
    /// Creates a planet given only a position and radius for use in simple scenes
    pub fn from_geometry(position: Vec3, radius: f32) -> Self {
        Planet {
            pose: Pose::new(position, Quat::IDENTITY),
            geometry: Sphere { radius },
            spin_speed: 0.,
        }
    }

    /// Get the planet's radius
    pub fn radius(&self) -> f32 {
        self.geometry.radius
    }

    /// Spin for a time dt in millisecond
    pub fn step(&mut self, dt: f32) {
        let delta = Quat::from_rotation_y(self.spin_speed * dt / 1000.);
        self.set_orientation((self.orientation() * delta).normalize());
    }

    /// Gives the albedo of a point (projected to the sphere) of the planet
    pub fn albedo(&self, point: Vec3) -> Vec3 {
        Vec3::new(1., 1., 1.)
    }
}
