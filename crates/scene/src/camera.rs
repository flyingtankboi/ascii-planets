
use crate::structures::Pose;

/// Represents a camera type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraType {

}


#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Position and orientation of the camera.
    /// see [`Pose`]
    pub pose : Pose,
    /// The camera type
    /// see [`CameraType`]
    pub light_type : CameraType,
}