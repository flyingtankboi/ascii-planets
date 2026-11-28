//! Camera module handling the camera object in a scene
use glam::{Quat, Vec3};

use crate::geometry::{HasPose, Pose};
use crate::planet::Planet;

/// Orbit speed in radian per second
const ORBIT_SPEED: f32 = 1.;
/// Zoom speed, the altitude is multiplied or divided by e every second
const ZOOM_SPEED: f32 = 1.;
/// Maximum pitch in radian, just under 90° to avoid flipping over the poles
const MAX_PITCH: f32 = 89f32.to_radians();
/// Default altitude above the target's surface, in target radii
const DEFAULT_ALTITUDE: f32 = 1.5;
/// Minimum altitude in target radii, zooming is multiplicative so this is only a safety floor
const MIN_ALTITUDE: f32 = 0.01;

/// Represents a camera type
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CameraType {
    /// Standard rays come from one point
    Perspective { fov_radian: f32 },
    /// Rays all come out parallel
    Orthogonal { ortho_height: f32 },
}

/// Movement directions requested for one step of the orbit camera
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OrbitDirection {
    /// Orbit up, over the target
    pub up: bool,
    /// Orbit down, under the target
    pub down: bool,
    /// Orbit to the left of the target
    pub left: bool,
    /// Orbit to the right of the target
    pub right: bool,
    /// Move closer to the target
    pub closer: bool,
    /// Move further from the target
    pub further: bool,
}

impl OrbitDirection {
    /// Gives 1, -1 or 0 depending on which of the two opposite directions is set
    fn axis(positive: bool, negative: bool) -> f32 {
        positive as i8 as f32 - negative as i8 as f32
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// Position and orientation of the camera.
    /// see [`Pose`]
    pub pose: Pose,
    /// The camera type
    /// see [`CameraType`]
    pub camera_type: CameraType,
    /// The planet around which it will orbit as the index in a scene's planet
    pub target_id: usize,
    /// Altitude above the target's surface in target radii.
    /// Keeps the same framing whatever the size of the target
    pub altitude: f32,
}

impl Camera {
    /// Create a new camera with a given camera type, you can then set a target to look at a certain planet
    pub fn new(camera_type: CameraType) -> Self {
        Camera {
            pose: Pose::default(),
            camera_type,
            target_id: 0,
            altitude: DEFAULT_ALTITUDE,
        }
    }

    /// Create a camera with a given pose and camera type, see [`Camera::new`] for the defaults
    pub fn with_pose(pose: Pose, camera_type: CameraType) -> Self {
        Camera {
            pose,
            camera_type,
            target_id: 0,
            altitude: DEFAULT_ALTITUDE,
        }
    }

    /// Sets the current target of the camera, keeps the orientation and the altitude so the
    /// new target is framed the same way, only moves position
    /// Call set_target on tab to change the id
    pub fn set_target(&mut self, target: &Planet, target_id: usize) {
        self.target_id = target_id;
        self.orbit(target, self.orientation());
    }

    /// Orbits around the target's vertical axis, positive yaw moves the camera to the left
    pub fn rotate_yaw(&mut self, target: &Planet, yaw: f32) {
        // Multiplying on the left rotates around the world Y axis, so the horizon stays level
        let orientation = Quat::from_rotation_y(yaw) * self.orientation();
        self.orbit(target, orientation);
    }

    /// Orbits over or under the target, positive pitch moves the camera up
    /// The pitch is clamped to [`MAX_PITCH`] so the camera never goes over the poles
    pub fn rotate_pitch(&mut self, target: &Planet, pitch: f32) {
        // Current pitch from the forward vector, positive when looking down
        let current = (-self.forward().y).clamp(-1., 1.).asin();
        let pitch = (current + pitch).clamp(-MAX_PITCH, MAX_PITCH) - current;
        // Multiplying on the right rotates around the camera's own X axis
        let orientation = self.orientation() * Quat::from_rotation_x(pitch);
        self.orbit(target, orientation);
    }

    /// Moves along the view direction by multiplying the altitude by `e^delta`,
    /// positive delta moves away from the target.
    /// Being multiplicative, zooming slows down close to the surface and never reaches it
    pub fn move_altitude(&mut self, target: &Planet, delta: f32) {
        self.altitude = (self.altitude * delta.exp()).max(MIN_ALTITUDE);
        self.orbit(target, self.orientation());
    }

    /// Moves the camera following the requested directions for a time dt in millisecond
    pub fn step(&mut self, target: &Planet, direction: &OrbitDirection, dt: f32) {
        let dt = dt / 1000.;
        let yaw = OrbitDirection::axis(direction.left, direction.right);
        let pitch = OrbitDirection::axis(direction.up, direction.down);
        let zoom = OrbitDirection::axis(direction.further, direction.closer);

        self.rotate_yaw(target, yaw * ORBIT_SPEED * dt);
        self.rotate_pitch(target, pitch * ORBIT_SPEED * dt);
        self.move_altitude(target, zoom * ZOOM_SPEED * dt);
    }

    /// Places the camera at its altitude above the target, looking at it with the given orientation
    fn orbit(&mut self, target: &Planet, orientation: Quat) {
        let orientation = orientation.normalize();
        let distance = target.radius() * (1. + self.altitude);
        let forward = orientation * Vec3::Z;
        self.pose = Pose::new(target.position() - distance * forward, orientation);
    }
}

impl HasPose for Camera {
    fn pose(&self) -> &Pose {
        &self.pose
    }
    fn pose_mut(&mut self) -> &mut Pose {
        &mut self.pose
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, LN_2};

    const EPSILON: f32 = 1e-4;

    fn assert_vec3_approx_eq(actual: Vec3, expected: Vec3) {
        assert!(
            (actual - expected).length() < EPSILON,
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    /// Camera orbiting a unit planet at z = 5, so 2.5 away from its center
    fn setup() -> (Camera, Planet) {
        let mut camera = Camera::new(CameraType::Perspective {
            fov_radian: FRAC_PI_2,
        });
        let planet = Planet::from_geometry(Vec3::new(0., 0., 5.), 1.);
        camera.set_target(&planet, 0);
        (camera, planet)
    }

    #[test]
    fn test_set_target_keeps_framing() {
        let (mut camera, _) = setup();
        assert_vec3_approx_eq(camera.position(), Vec3::new(0., 0., 2.5));

        // Same altitude in radii, so a planet three times bigger is three times further
        let big = Planet::from_geometry(Vec3::new(10., 0., 0.), 3.);
        camera.set_target(&big, 1);
        assert_eq!(camera.target_id, 1);
        assert!((camera.distance(&big) - 7.5).abs() < EPSILON);
        assert_vec3_approx_eq(camera.forward(), Vec3::Z);
    }

    #[test]
    fn test_yaw_keeps_distance_and_looks_at_target() {
        let (mut camera, planet) = setup();
        camera.rotate_yaw(&planet, FRAC_PI_2);

        // Positive yaw moves the camera to the left (-X) of the planet
        assert_vec3_approx_eq(camera.position(), Vec3::new(-2.5, 0., 5.));
        assert_vec3_approx_eq(camera.forward(), Vec3::X);
        // The horizon stays level
        assert!(camera.right().y.abs() < EPSILON);
    }

    #[test]
    fn test_pitch_moves_up_and_is_clamped() {
        let (mut camera, planet) = setup();
        camera.rotate_pitch(&planet, 0.1);
        assert!(camera.position().y > 0.);
        assert!(camera.forward().y < 0.);

        camera.rotate_pitch(&planet, 10.);
        let pitch = (-camera.forward().y).asin();
        assert!((pitch - MAX_PITCH).abs() < EPSILON);
    }

    #[test]
    fn test_move_altitude_is_multiplicative_and_stays_outside() {
        let (mut camera, planet) = setup();
        // e^ln2 doubles the altitude: 1.5 -> 3 radii, so 4 away from the center
        camera.move_altitude(&planet, LN_2);
        assert!((camera.distance(&planet) - 4.).abs() < EPSILON);

        camera.move_altitude(&planet, -100.);
        assert_eq!(camera.altitude, MIN_ALTITUDE);
        assert!(camera.distance(&planet) > planet.radius());
    }

    #[test]
    fn test_step_without_direction_does_not_move() {
        let (mut camera, planet) = setup();
        let before = camera.pose;
        camera.step(&planet, &OrbitDirection::default(), 16.);
        assert_vec3_approx_eq(camera.position(), before.position);
    }
}
