use crate::math::{Direction3, Point3, Ray};
use crate::scene::{Sphere, Triangle};

const TRIANGLE_DETERMINANT_EPSILON: f64 = 1.0e-12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitObject {
    Sphere(usize),
    Triangle(usize),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SurfaceHit {
    pub distance: f64,
    pub point: Point3,
    pub outward_normal: Direction3,
    pub hit_object: HitObject,
}

pub fn closest_scene_hit(
    ray: Ray,
    spheres: &[Sphere],
    triangles: &[Triangle],
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

    for (triangle_index, triangle) in triangles.iter().enumerate() {
        if let Some(hit) =
            intersect_triangle(ray, triangle, triangle_index, t_min, closest_distance)
        {
            closest_distance = hit.distance;
            closest_hit = Some(hit);
        }
    }

    closest_hit
}

pub fn any_scene_hit(
    ray: Ray,
    spheres: &[Sphere],
    triangles: &[Triangle],
    t_min: f64,
    t_max: f64,
    excluded_object: Option<HitObject>,
) -> bool {
    for (sphere_index, sphere) in spheres.iter().enumerate() {
        if excluded_object == Some(HitObject::Sphere(sphere_index)) {
            continue;
        }

        if intersect_sphere(ray, sphere, sphere_index, t_min, t_max).is_some() {
            return true;
        }
    }

    for (triangle_index, triangle) in triangles.iter().enumerate() {
        if excluded_object == Some(HitObject::Triangle(triangle_index)) {
            continue;
        }

        if intersect_triangle(ray, triangle, triangle_index, t_min, t_max).is_some() {
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
        hit_object: HitObject::Sphere(sphere_index),
    })
}

fn intersect_triangle(
    ray: Ray,
    triangle: &Triangle,
    triangle_index: usize,
    t_min: f64,
    t_max: f64,
) -> Option<SurfaceHit> {
    // Moller-Trumbore ray-triangle intersection with no backface culling.
    let vertex0 = triangle.vertex0();
    let edge01 = triangle.vertex1() - vertex0;
    let edge02 = triangle.vertex2() - vertex0;

    let pvec = ray.direction.cross(edge02);
    let determinant = edge01.dot(pvec);
    if determinant.abs() < TRIANGLE_DETERMINANT_EPSILON {
        return None;
    }

    let inverse_determinant = 1.0 / determinant;
    let tvec = ray.origin - vertex0;
    let barycentric_u = tvec.dot(pvec) * inverse_determinant;
    if !(0.0..=1.0).contains(&barycentric_u) {
        return None;
    }

    let qvec = tvec.cross(edge01);
    let barycentric_v = ray.direction.dot(qvec) * inverse_determinant;
    if barycentric_v < 0.0 || barycentric_u + barycentric_v > 1.0 {
        return None;
    }

    let distance = edge02.dot(qvec) * inverse_determinant;
    if distance < t_min || distance > t_max {
        return None;
    }

    let point = ray.at(distance);
    let geometric_normal = edge01.cross(edge02).normalized();
    let outward_normal = if geometric_normal.dot(ray.direction) <= 0.0 {
        geometric_normal
    } else {
        -geometric_normal
    };

    Some(SurfaceHit {
        distance,
        point,
        outward_normal,
        hit_object: HitObject::Triangle(triangle_index),
    })
}

#[cfg(test)]
mod tests {
    use crate::math::{Point3, Ray, Vec3};
    use crate::scene::{Material, Sphere, Triangle};

    use super::{HitObject, any_scene_hit, closest_scene_hit};

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

    fn test_triangle(vertex0: Point3, vertex1: Point3, vertex2: Point3) -> Triangle {
        Triangle::try_new(vertex0, vertex1, vertex2, Material::default())
            .expect("test triangle values should be valid")
    }

    #[test]
    fn ray_hits_sphere_and_reports_expected_distance_and_normal() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.4);
        let hit = closest_scene_hit(ray, &[sphere], &[], 1.0e-6, f64::INFINITY)
            .expect("ray should hit sphere");

        assert_eq!(hit.hit_object, HitObject::Sphere(0));
        assert_nearly_equal(hit.distance, 0.6);
        assert_nearly_equal(hit.point.z, 0.4);
        assert_nearly_equal(hit.outward_normal.x, 0.0);
        assert_nearly_equal(hit.outward_normal.y, 0.0);
        assert_nearly_equal(hit.outward_normal.z, 1.0);
    }

    #[test]
    fn ray_hits_triangle_and_reports_expected_distance_and_normal() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let triangle = test_triangle(
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );
        let hit = closest_scene_hit(ray, &[], &[triangle], 1.0e-6, f64::INFINITY)
            .expect("ray should hit triangle");

        assert_eq!(hit.hit_object, HitObject::Triangle(0));
        assert_nearly_equal(hit.distance, 1.0);
        assert_nearly_equal(hit.point.x, 0.0);
        assert_nearly_equal(hit.point.y, 0.0);
        assert_nearly_equal(hit.point.z, 0.0);
        assert_nearly_equal(hit.outward_normal.x, 0.0);
        assert_nearly_equal(hit.outward_normal.y, 0.0);
        assert_nearly_equal(hit.outward_normal.z, 1.0);
    }

    #[test]
    fn ray_misses_triangle_returns_none() {
        let ray = Ray::new(Point3::new(0.0, 0.9, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let triangle = test_triangle(
            Point3::new(-0.5, -0.5, 0.0),
            Point3::new(0.5, -0.5, 0.0),
            Point3::new(0.0, 0.5, 0.0),
        );

        let hit = closest_scene_hit(ray, &[], &[triangle], 1.0e-6, f64::INFINITY);
        assert!(hit.is_none());
    }

    #[test]
    fn ray_miss_returns_none() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 1.0, 0.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.4);
        let triangle = test_triangle(
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );

        let hit = closest_scene_hit(ray, &[sphere], &[triangle], 1.0e-6, f64::INFINITY);
        assert!(hit.is_none());
    }

    #[test]
    fn closest_hit_picks_nearest_object_across_spheres_and_triangles() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, -1.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.5);
        let triangle = test_triangle(
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, -1.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
        );

        let hit = closest_scene_hit(ray, &[sphere], &[triangle], 1.0e-6, f64::INFINITY)
            .expect("ray should hit both objects");

        assert_eq!(hit.hit_object, HitObject::Triangle(0));
        assert_nearly_equal(hit.distance, 1.0);
    }

    #[test]
    fn hit_outside_t_interval_is_rejected() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, -1.0));
        let triangle = test_triangle(
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, -1.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
        );

        let hit = closest_scene_hit(ray, &[], &[triangle], 0.0, 0.5);
        assert!(hit.is_none());
    }

    #[test]
    fn any_hit_respects_excluded_object() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 2.0), Vec3::new(0.0, 0.0, -1.0));
        let sphere = test_sphere(Point3::new(0.0, 0.0, 0.0), 0.5);
        let triangle = test_triangle(
            Point3::new(-1.0, -1.0, 1.0),
            Point3::new(1.0, -1.0, 1.0),
            Point3::new(0.0, 1.0, 1.0),
        );

        let has_hit_without_exclusion = any_scene_hit(
            ray,
            &[sphere.clone()],
            &[triangle.clone()],
            1.0e-6,
            f64::INFINITY,
            None,
        );
        let has_hit_with_sphere_excluded = any_scene_hit(
            ray,
            &[sphere.clone()],
            &[triangle.clone()],
            1.0e-6,
            f64::INFINITY,
            Some(HitObject::Sphere(0)),
        );
        let has_hit_with_triangle_excluded = any_scene_hit(
            ray,
            &[sphere],
            &[triangle],
            1.0e-6,
            f64::INFINITY,
            Some(HitObject::Triangle(0)),
        );

        assert!(has_hit_without_exclusion);
        assert!(has_hit_with_sphere_excluded);
        assert!(has_hit_with_triangle_excluded);
    }

    #[test]
    fn any_hit_is_false_when_only_hit_object_is_excluded() {
        let ray = Ray::new(Point3::new(0.0, 0.0, 1.0), Vec3::new(0.0, 0.0, -1.0));
        let triangle = test_triangle(
            Point3::new(-1.0, -1.0, 0.0),
            Point3::new(1.0, -1.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
        );

        let has_hit = any_scene_hit(
            ray,
            &[],
            &[triangle],
            1.0e-6,
            f64::INFINITY,
            Some(HitObject::Triangle(0)),
        );

        assert!(!has_hit);
    }
}
