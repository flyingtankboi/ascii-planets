use crate::ray::Ray;

use glam::Vec3;
use scene::{geometry::HasPose, planet::Planet};

/// Represents a hit between a planet and a ray
#[derive(Debug, Clone)]
pub struct Hit<'a> {
    /// Distance between hit point and ray origin
    pub distance: f32,
    /// Hit point position
    pub point: Vec3,
    /// Normal of the surface at hit point
    pub normal: Vec3,
    /// Which planet was hit
    pub planet: &'a Planet,
}

fn intersect_with_one(ray: Ray, planet: &Planet, t_min: f32, t_max: f32) -> Option<Vec3> {
    let l = ray.origin - planet.position();

    // Using half_b simplifies the quadratic formula by eliminating factors of 2 and 4
    let a = ray.direction.dot(ray.direction);
    let half_b = l.dot(ray.direction);
    let c = l.dot(l) - planet.radius() * planet.radius();

    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_d = discriminant.sqrt();

    // Find the nearest root that lies in the acceptable range [t_min, t_max]
    let mut root = (-half_b - sqrt_d) / a;
    if root < t_min || root > t_max {
        root = (-half_b + sqrt_d) / a;
        if root < t_min || root > t_max {
            return None;
        }
    }

    let point = ray.at(root);
    Some(point)
}

pub fn intersect<'a>(ray: Ray, planets: &'a [Planet]) -> Option<Hit<'a>> {
    planets
        .iter()
        .filter_map(|planet| {
            let point = intersect_with_one(ray, planet, 0.001, f32::MAX)?;
            Some(Hit {
                distance: point.distance(ray.origin),
                point,
                normal: (point - planet.position()) * (1.0 / planet.radius()),
                planet,
            })
        })
        .min_by(|hita, hitb| f32::total_cmp(&hita.distance, &hitb.distance))
}

#[cfg(test)]
mod tests {
    use super::*;

    const EPSILON: f32 = 1e-5;

    fn assert_vec3_approx_eq(actual: Vec3, expected: Vec3) {
        assert!(
            (actual - expected).length() < EPSILON,
            "expected {:?}, got {:?}",
            expected,
            actual
        );
    }

    #[test]
    fn test_ray_misses_planet() {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let planet = Planet::from_geometry(Vec3::new(10.0, 0.0, 5.0), 1.0);
        let planets = [planet];

        assert!(intersect_with_one(ray, &planets[0], 0.001, f32::MAX).is_none());
        assert!(intersect(ray, &planets).is_none());
    }

    #[test]
    fn test_ray_hits_planet_front_surface() {
        // Ray at origin pointing +Z toward a sphere at z = 5 with radius 1
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let planets = [Planet::from_geometry(Vec3::new(0.0, 0.0, 5.0), 1.0)];

        let hit = intersect(ray, &planets).expect("expected a hit");

        assert!((hit.distance - 4.0).abs() < EPSILON);
        assert_vec3_approx_eq(hit.point, Vec3::new(0.0, 0.0, 4.0));
        assert_vec3_approx_eq(hit.normal, Vec3::new(0.0, 0.0, -1.0));
    }

    #[test]
    fn test_ray_starts_inside_planet() {
        // Ray starts at the center of the sphere; first root is negative (< t_min),
        // so it should pick the exit point at z = 2.0
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let planets = [Planet::from_geometry(Vec3::ZERO, 2.0)];

        let hit = intersect(ray, &planets).expect("expected exit hit");

        assert!((hit.distance - 2.0).abs() < EPSILON);
        assert_vec3_approx_eq(hit.point, Vec3::new(0.0, 0.0, 2.0));
        assert_vec3_approx_eq(hit.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn test_planet_behind_ray_is_ignored() {
        // Planet is behind the ray (-Z), while ray shoots toward +Z
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let planets = [Planet::from_geometry(Vec3::new(0.0, 0.0, -5.0), 1.0)];

        assert!(intersect(ray, &planets).is_none());
    }

    #[test]
    fn test_t_min_and_t_max_bounds() {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        let planet = Planet::from_geometry(Vec3::new(0.0, 0.0, 5.0), 1.0);

        // Hit is at t = 4.0 and t = 6.0; restricting t_max to 3.0 should miss
        assert!(intersect_with_one(ray, &planet, 0.001, 3.0).is_none());

        // Setting t_min = 4.5 skips the front hit (4.0) and hits the back surface (6.0)
        let back_hit =
            intersect_with_one(ray, &planet, 4.5, f32::MAX).expect("expected back-face hit");
        assert_vec3_approx_eq(back_hit, Vec3::new(0.0, 0.0, 6.0));
    }

    #[test]
    fn test_closest_planet_wins_with_multiple_planets() {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);

        // Closer, larger planet (front surface at z = 3.0)
        let near_planet = Planet::from_geometry(Vec3::new(0.0, 0.0, 5.0), 2.0);
        // Farther, smaller planet (front surface at z = 9.5)
        let far_planet = Planet::from_geometry(Vec3::new(0.0, 0.0, 10.0), 0.5);

        let planets = [far_planet, near_planet];
        let hit = intersect(ray, &planets).expect("expected closest planet hit");

        assert!((hit.distance - 3.0).abs() < EPSILON);
        assert_vec3_approx_eq(hit.point, Vec3::new(0.0, 0.0, 3.0));
    }

    #[test]
    fn test_empty_planet_slice() {
        let ray = Ray::new(Vec3::ZERO, Vec3::Z);
        assert!(intersect(ray, &[]).is_none());
    }

    #[test]
    fn test_non_unit_ray_direction() {
        // Ray direction has length 2.0; hit point and distance should remain unaffected
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, 2.0));
        let planets = [Planet::from_geometry(Vec3::new(0.0, 0.0, 5.0), 1.0)];

        let hit = intersect(ray, &planets).expect("expected hit");

        assert!((hit.distance - 4.0).abs() < EPSILON);
        assert_vec3_approx_eq(hit.point, Vec3::new(0.0, 0.0, 4.0));
    }
}
