use glam::Vec3;

/// Ray structure
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ray {
    /// Ray's origin
    pub origin: Vec3,
    /// Ray's direction
    pub direction: Vec3,
}

impl Default for Ray {
    fn default() -> Self {
        Ray {
            origin : Vec3::ZERO,
            direction : Vec3::Z
        }
    }
}