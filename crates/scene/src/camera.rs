//! Camera module handling the camera object in a scene
use crate::geometry::{HasPose, Pose};

/// Represents a camera type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraType {
    /// Standard rays come from one point
    Perspective { fov_radian: f32 },
    /// Rays all come out parallel
    Orthogonal { ortho_height: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Position and orientation of the camera.
    /// see [`Pose`]
    pub pose: Pose,
    /// The camera type
    /// see [`CameraType`]
    pub camera_type: CameraType,
}

impl HasPose for Camera {
    fn pose(&self) -> &Pose {
        &self.pose
    }
}
