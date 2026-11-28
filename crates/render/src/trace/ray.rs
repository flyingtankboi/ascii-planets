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

    /// Creates a ray through the screen position `(u, v)`, both in `[0, 1]` from the top left.
    /// `cell_aspect` is the height / width ratio of one cell
    pub fn from_camera_at_uv(
        camera: &Camera,
        u: f32,
        v: f32,
        width: usize,
        height: usize,
        cell_aspect: f32,
    ) -> Ray {
        let x_screen = 2. * u - 1.;
        let y_screen = 1. - 2. * v; // Flip y so y points up and not down as is the case in image formats

        // Cells are taller than wide, so the canvas is physically `height * cell_aspect` tall
        let aspect_ratio = width as f32 / (height as f32 * cell_aspect);
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

        Ray::new(origin, direction)
    }

    /// Get position of ray projected at distance t
    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::grid::Grid;
    use scene::geometry::Pose;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4};

    const EPSILON: f32 = 1e-5;

    fn assert_vec3_approx_eq(actual: Vec3, expected: Vec3) {
        assert!(
            (actual - expected).length() < EPSILON,
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    /// Ray through the center of cell `(x, y)` with 2:1 cells
    fn center_ray(cam: &Camera, x: usize, y: usize, width: usize, height: usize) -> Ray {
        let u = (x as f32 + 0.5) / width as f32;
        let v = (y as f32 + 0.5) / height as f32;
        Ray::from_camera_at_uv(cam, u, v, width, height, 2.)
    }

    /// Rays through the center of every cell
    fn center_rays(cam: &Camera, width: usize, height: usize) -> Grid<Ray> {
        Grid::new_with(width, height, |x, y| center_ray(cam, x, y, width, height))
    }

    fn print_rays(rays: &Grid<Ray>, width: usize, height: usize) {
        let axes = [Vec3::Z, Vec3::Y, Vec3::X];
        let ranges: Vec<(f32, f32)> = axes
            .iter()
            .map(|&axis| {
                let min = rays
                    .iter()
                    .map(|(_, _, r)| r.direction.dot(axis))
                    .min_by(f32::total_cmp)
                    .unwrap();
                let max = rays
                    .iter()
                    .map(|(_, _, r)| r.direction.dot(axis))
                    .max_by(f32::total_cmp)
                    .unwrap();
                (min, max)
            })
            .collect();

        for y in 0..height {
            for (axis, (min, max)) in axes.iter().zip(&ranges) {
                for x in 0..width {
                    let val = rays[(x, y)].direction.dot(*axis);
                    let norm = if (max - min).abs() < EPSILON {
                        0.0
                    } else {
                        (val - min) / (max - min) * 9.0
                    };
                    print!("{norm:.0}");
                }
                print!("  ");
            }
            println!();
        }
        println!();
    }

    #[test]
    fn test_print_rays() {
        let (width, height) = (24, 12);
        let camera_types = [
            Perspective {
                fov_radian: FRAC_PI_2,
            },
            Perspective {
                fov_radian: FRAC_PI_4,
            },
            Orthogonal { ortho_height: 5.0 },
        ];

        for camera_type in camera_types {
            let cam = Camera::new(camera_type);
            let rays = center_rays(&cam, width, height);
            print_rays(&rays, width, height);
        }
    }

    #[test]
    fn test_perspective_rays() {
        let cam = Camera::new(Perspective {
            fov_radian: FRAC_PI_2,
        });

        // Center cell (1, 1) on a 3x3 grid shoots straight forward
        let center = center_ray(&cam, 1, 1, 3, 3);
        assert_vec3_approx_eq(center.origin, cam.pose.position);
        assert_vec3_approx_eq(center.direction, cam.forward().normalize());

        // Corner math on 4x2 grid with 2:1 cells (effective height = 4, aspect_ratio = 1.0, scale = 1.0)
        let top_left = center_ray(&cam, 0, 0, 4, 2);
        let expected_dir = (cam.forward() + cam.right() * -0.75 + cam.up() * 0.50).normalize();
        assert_vec3_approx_eq(top_left.direction, expected_dir);
    }

    #[test]
    fn test_orthogonal_rays_and_at() {
        let cam = Camera::new(Orthogonal { ortho_height: 4.0 });

        // Orthogonal rays are parallel with shifted origins
        let ray_tl = center_ray(&cam, 0, 0, 4, 2);
        let ray_br = center_ray(&cam, 3, 1, 4, 2);

        assert_vec3_approx_eq(ray_tl.direction, cam.forward().normalize());
        assert_vec3_approx_eq(ray_br.direction, cam.forward().normalize());
        assert_vec3_approx_eq(
            ray_tl.origin,
            cam.pose.position + (-1.5 * cam.right()) + (1.0 * cam.up()),
        );

        // Ray::at projects along direction
        assert_vec3_approx_eq(ray_tl.at(3.0), ray_tl.origin + ray_tl.direction * 3.0);
    }

    #[test]
    fn test_from_camera_dimensions_and_symmetry() {
        let cam = Camera::new(Perspective {
            fov_radian: FRAC_PI_2,
        });
        let rays = center_rays(&cam, 4, 2);
        assert_eq!((rays.width, rays.height), (4, 2));

        // Left/right and top/bottom rays mirror each other
        let tl = rays[(0, 0)].direction;
        let tr = rays[(3, 0)].direction;
        let bl = rays[(0, 1)].direction;
        assert_vec3_approx_eq(tr, Vec3::new(-tl.x, tl.y, tl.z));
        assert_vec3_approx_eq(bl, Vec3::new(tl.x, -tl.y, tl.z));
    }

    #[test]
    fn test_rotated_camera() {
        // Rotating 90° around Y turns forward from +Z to +X
        let cam = Camera::with_pose(
            Pose::new(Vec3::ZERO, glam::Quat::from_rotation_y(FRAC_PI_2)),
            Perspective {
                fov_radian: FRAC_PI_2,
            },
        );
        let center = center_ray(&cam, 1, 1, 3, 3);
        assert_vec3_approx_eq(center.direction, Vec3::X);
    }
}
