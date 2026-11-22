//! Common structures to scene objects.

use glam::{Quat, Vec3};

/// Represents a 3D position and orientation in space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pose {
    /// Position in 3D space (x, y, z).
    /// see [`Vec3`]
    pub position: Vec3,
    /// Orientation represented as a unit quaternion
    /// see [`Quat`]
    pub orientation: Quat,
}

impl Default for Pose {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            orientation: Quat::IDENTITY,
        }
    }
}

impl Pose {
    pub fn new(position: Vec3, orientation: Quat) -> Self {
        Pose {
            position,
            orientation,
        }
    }
}

/// A general trait to give to all objects with a pose field.
/// Enables handling the calculations for position and rotation easily.
pub trait HasPose {
    fn pose(&self) -> &Pose;

    fn position(&self) -> Vec3 {
        self.pose().position
    }

    fn orientation(&self) -> Quat {
        self.pose().orientation
    }

    fn distance(&self, other: &impl HasPose) -> f32 {
        self.pose().position.distance(other.pose().position)
    }

    fn up(&self) -> Vec3 {
        self.pose().orientation * Vec3::Y
    }

    fn right(&self) -> Vec3 {
        self.pose().orientation * Vec3::X
    }

    fn forward(&self) -> Vec3 {
        self.pose().orientation * Vec3::Z
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A simple sphere geometry
pub struct Sphere {
    /// Radius of the sphere
    pub radius: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_pose() {
        let default = Pose::default();
        assert_eq!(default.position, Vec3::ZERO);
        assert_eq!(default.orientation, Quat::IDENTITY);
    }
}
