//! Light module handling the light object in a scene
use crate::geometry::{HasPose, Pose};
use glam::Vec3;

/// Represents a light type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LightType {
    /// Global light source with all parallel rays.
    DirectionalLight,
    /// Light at a physical point.
    PointLight,
    /// Light at a physical point with an emitting cone defined by an angle.
    SpotLight { angle: f32 },
    /// Ambient light coming from everywhere
    AmbientLight,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Light {
    /// Position and orientation of the light source.
    /// see [`Pose`]
    pub pose: Pose,
    /// The light type
    /// see [`LightType`]
    pub light_type: LightType,
    /// Light intensity for Red Green Blue i.e an unbounded energy value
    pub color: Vec3,
}

impl HasPose for Light {
    fn pose(&self) -> &Pose {
        &self.pose
    }
    fn pose_mut(&mut self) -> &mut Pose {
        &mut self.pose
    }
}
