use crate::math::{Direction3, Point3, Ray};
use crate::scene::Sphere;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceHit {
    pub distance: f64,
    pub point: Point3,
    pub outward_normal: Direction3,
    pub sphere_index: usize,
}

pub fn closest_sphere_hit(
    ray: Ray,
    spheres: &[Sphere],
    t_min: f64,
    t_max: f64,
) -> Option<SurfaceHit> {
    let mut closest_distance = t_max;
    let mut closest_hit: Option<SurfaceHit> = None;

    for (sphere_index, sphere) in spheres.iter().enumerate() {
        if let Some(hit) = intersect_sphere(ray, sphere, sphere_index, t_min, closest_distance) {
            closest_distance = hit.distance;
            closest_hit = Some(hit);
        }
    }

    closest_hit
}

pub fn any_sphere_hit(
    ray: Ray,
    spheres: &[Sphere],
    t_min: f64,
    t_max: f64,
    excluded_sphere_index: Option<usize>,
) -> bool {
    for (sphere_index, sphere) in spheres.iter().enumerate() {
        if excluded_sphere_index == Some(sphere_index) {
            continue;
        }

        if intersect_sphere(ray, sphere, sphere_index, t_min, t_max).is_some() {
            return true;
        }
    }

    false
}

fn intersect_sphere(
    ray: Ray,
    sphere: &Sphere,
    sphere_index: usize,
    t_min: f64,
    t_max: f64,
) -> Option<SurfaceHit> {
    let center_to_ray_origin = ray.origin - sphere.center();
    let a = ray.direction.length_squared();
    let half_b = ray.direction.dot(center_to_ray_origin);
    let c = center_to_ray_origin.length_squared() - sphere.radius() * sphere.radius();

    let discriminant = half_b * half_b - a * c;
    if discriminant < 0.0 {
        return None;
    }

    let sqrt_discriminant = discriminant.sqrt();
    let near_root = (-half_b - sqrt_discriminant) / a;
    let far_root = (-half_b + sqrt_discriminant) / a;

    let distance = if near_root >= t_min && near_root <= t_max {
        near_root
    } else if far_root >= t_min && far_root <= t_max {
        far_root
    } else {
        return None;
    };

    let point = ray.at(distance);
    let outward_normal = (point - sphere.center()) / sphere.radius();

    Some(SurfaceHit {
        distance,
        point,
        outward_normal,
        sphere_index,
    })
}

#[cfg(test)]
mod tests {
    use crate::math::{Point3, Ray, Vec3};
    use crate::scene::{Material, Sphere};

    use super::{any_sphere_hit, closest_sphere_hit};

    const EPSILON: f64 = 1.0e-12;

    fn assert_nearly_equal(lhs: f64, rhs: f64) {
        assert!(
            (lhs - rhs).abs() < EPSILON,
            "expected values to be nearly equal: lhs={lhs}, rhs={rhs}"
        );
    }

    fn test_sphere(center: Point3, radius: f64) -> Sphere {
        Sphere::try_new(center, radius, Material::default())
            .expect("test sphere values should be valid")
    }

    #[test]
    fn ray_hits_sphere_and_reports_expected_distance_and_normal() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.4);
        let hit = closest_sphere_hit(ray, &[sphere], 1.0e-6, f64::INFINITY)
            .expect("ray should hit sphere");

        assert_nearly_equal(hit.distance, 0.6);
        assert_nearly_equal(hit.point.z, 0.4);
        assert_nearly_equal(hit.outward_normal.x, 0.0);
        assert_nearly_equal(hit.outward_normal.y, 0.0);
        assert_nearly_equal(hit.outward_normal.z, 1.0);
    }

    #[test]
    fn ray_miss_returns_none() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.4);

        let hit = closest_sphere_hit(ray, &[sphere], 1.0e-6, f64::INFINITY);
        assert!(hit.is_none());
    }

    #[test]
    fn closest_hit_picks_nearest_sphere() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let near_sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.2);
        let far_sphere = test_sphere(Point3::new(0.0, 0.0, -1.0), 0.2);

        let hit = closest_sphere_hit(ray, &[far_sphere, near_sphere], 1.0e-6, f64::INFINITY)
            .expect("ray should hit both spheres");

        assert_eq!(hit.sphere_index, 1);
        assert_nearly_equal(hit.distance, 0.8);
    }

    #[test]
    fn hit_outside_t_interval_is_rejected() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.4);

        let hit = closest_sphere_hit(ray, &[sphere], 0.0, 0.5);
        assert!(hit.is_none());
    }

    #[test]
    fn any_hit_respects_excluded_sphere_index() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.4);

        let has_hit_without_exclusion =
            any_sphere_hit(ray, &[sphere.clone()], 1.0e-6, f64::INFINITY, None);
        let has_hit_with_exclusion = any_sphere_hit(ray, &[sphere], 1.0e-6, f64::INFINITY, Some(0));

        assert!(has_hit_without_exclusion);
        assert!(!has_hit_with_exclusion);
    }
}
