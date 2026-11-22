use glam::Vec3;
use scene::camera::{Camera, CameraType::Orthogonal, CameraType::Perspective};
use scene::geometry::HasPose;

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
            origin: Vec3::ZERO,
            direction: Vec3::Z,
        }
    }
}

impl Ray {
    /// Creates a new ray instance
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Ray { origin, direction }
    }

    fn idx(x: usize, y: usize, width: usize, height: usize) -> Option<usize> {
        (x < width && y < height).then_some(y * width + x)
    }

    /// Creates a ray from a cell's position and a camera
    /// If the position is outside of width/height return none
    pub fn from_camera_at_point(
        camera: Camera,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> Option<Ray> {
        Ray::idx(x, y, width, height)?;

        let height = height * 2; // Actual height is double because ascii is 2:1

        let u = (x as f32 + 0.5) / width as f32;
        let v = (2. * y as f32 + 1.) / height as f32; // Use +1 instead of +0.5 to get middle as Cells are 2:1

        let x_screen = 2. * u - 1.;
        let y_screen = 1. - 2. * v; // Flip y so y points up and not down as is the case in image formats

        let aspect_ratio = width as f32 / height as f32;
        let scale = match camera.camera_type {
            Perspective { fov_radian } => (fov_radian * 0.5).tan(),
            Orthogonal { ortho_height: _ } => 1.,
        };

        let x_cam = x_screen * aspect_ratio * scale;
        let y_cam = y_screen * scale;

        let direction = match camera.camera_type {
            Perspective { fov_radian: _ } => {
                (camera.forward() + camera.right() * x_cam + camera.up() * y_cam).normalize()
            }
            Orthogonal { ortho_height: _ } => camera.forward().normalize(),
        };

        let origin = match camera.camera_type {
            Perspective { fov_radian: _ } => camera.pose.position,
            Orthogonal { ortho_height } => {
                camera.pose.position
                    + (x_screen * aspect_ratio * ortho_height * 0.5) * camera.right()
                    + (y_screen * ortho_height * 0.5) * camera.up()
            }
        };

        Some(Ray::new(origin, direction))
    }

    /// Get all rays for a camera and for the width/height of the canvas
    pub fn from_camera(camera: Camera, width: usize, height: usize) -> Vec<Ray> {
        let mut ray_array = Vec::new();
        for y in 0..height {
            for x in 0..width {
                ray_array.push(
                    Ray::from_camera_at_point(camera, x, y, width, height).unwrap_or_default(),
                )
            }
        }
        ray_array
    }

    /// Get position of ray projected at distance t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use glam::Vec3;
    use scene::geometry::Pose;

    fn print_rays(rays: Vec<Ray>, width: usize, height: usize) {
        let min_z = rays
            .iter()
            .map(|r| r.direction.dot(Vec3::Z))
            .min_by(f32::total_cmp)
            .unwrap();
        let max_z = rays
            .iter()
            .map(|r| r.direction.dot(Vec3::Z))
            .max_by(f32::total_cmp)
            .unwrap();

        let min_y = rays
            .iter()
            .map(|r| r.direction.dot(Vec3::Y))
            .min_by(f32::total_cmp)
            .unwrap();
        let max_y = rays
            .iter()
            .map(|r| r.direction.dot(Vec3::Y))
            .max_by(f32::total_cmp)
            .unwrap();

        let min_x = rays
            .iter()
            .map(|r| r.direction.dot(Vec3::X))
            .min_by(f32::total_cmp)
            .unwrap();
        let max_x = rays
            .iter()
            .map(|r| r.direction.dot(Vec3::X))
            .max_by(f32::total_cmp)
            .unwrap();

        for y in 0..height {
            for x in 0..width {
                let r = (rays[y * width + x].direction.dot(Vec3::Z) - min_z) * 1. / (max_z - min_z)
                    * 9.;
                print!("{r:.0?}")
            }
            print!("  ");
            for x in 0..width {
                let r = (rays[y * width + x].direction.dot(Vec3::Y) - min_y) * 1. / (max_y - min_y)
                    * 9.;
                print!("{r:.0?}")
            }
            print!("  ");
            for x in 0..width {
                let r = (rays[y * width + x].direction.dot(Vec3::X) - min_x) * 1. / (max_x - min_x)
                    * 9.;
                print!("{r:.0?}")
            }
            print!("  ");
            println!();
        }
        println!();
    }

    #[test]
    fn test_print_rays() {
        let (width, height) = (24, 12);

        let cam = Camera {
            pose: Pose::default(),
            camera_type: Perspective {
                fov_radian: (std::f64::consts::PI as f32) / 2.,
            },
        };
        let rays = Ray::from_camera(cam, width, height);
        print_rays(rays, width, height);
    }
}
