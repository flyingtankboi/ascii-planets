use crate::structures::{Pose, Sphere};



// Represents the planets in the scene
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Planet{
    /// Position in 3D space (x, y, z).
    /// see [`Vec3`]
    pose : Pose,
    // The simple geometric form used
    pub geometry : Sphere,
}