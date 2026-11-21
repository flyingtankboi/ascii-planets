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

#[derive(Debug, Clone, Copy, PartialEq)]
/// A simple sphere geometry
pub struct Sphere {
    /// Radius of the sphere
    radius : u32,
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