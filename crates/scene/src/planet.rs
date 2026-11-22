use glam::{Quat, Vec3};

use crate::geometry::{HasPose, Pose, Sphere};

// Represents the planets in the scene
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planet {
    /// Pose in 3D space.
    /// see [`Pose`]
    pub pose: Pose,
    // The simple geometric form used
    pub geometry: Sphere,
}

impl HasPose for Planet {
    fn pose(&self) -> &Pose {
        &self.pose
    }
}

impl Planet {
    pub fn from_geometry(position: Vec3, radius: f32) -> Self {
        Planet {
            pose: Pose::new(position, Quat::IDENTITY),
            geometry: Sphere { radius },
        }
    }

    pub fn radius(&self) -> f32 {
        self.geometry.radius
    }
}
