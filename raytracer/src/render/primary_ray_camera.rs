use crate::math::{Direction3, Point3, Ray};
use crate::scene::Camera;

// Course reference images use a tighter projection than the textbook
// 2*tan(vfov/2) viewport formula.
const REFERENCE_VIEWPORT_HEIGHT_SCALE: f64 = 1.125;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrimaryRayCamera {
    camera_origin: Point3,
    pixel00_world: Point3,
    pixel_delta_u: Direction3,
    pixel_delta_v: Direction3,
}

impl PrimaryRayCamera {
    pub fn from_scene_camera(camera: &Camera, image_width: u32, image_height: u32) -> Self {
        assert!(image_width > 0, "image width must be greater than zero");
        assert!(image_height > 0, "image height must be greater than zero");

        let aspect_ratio = image_width as f64 / image_height as f64;
        let look_from = camera.look_from();
        let look_at = camera.look_at();
        let look_up = camera.look_up();

        let focal_length = (look_from - look_at).length();
        let theta = camera.field_of_view_degrees().to_radians();
        let viewport_height = REFERENCE_VIEWPORT_HEIGHT_SCALE * (theta * 0.5).tan() * focal_length;
        let viewport_width = viewport_height * aspect_ratio;

        let w = (look_from - look_at).normalized();
        let u = look_up.cross(w).normalized();
        let v = w.cross(u);

        let viewport_u = viewport_width * u;
        let viewport_v = -viewport_height * v;

        let pixel_delta_u = viewport_u / image_width as f64;
        let pixel_delta_v = viewport_v / image_height as f64;

        let viewport_upper_left =
            look_from - (focal_length * w) - (viewport_u * 0.5) - (viewport_v * 0.5);
        let pixel00_world = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            camera_origin: look_from,
            pixel00_world,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    pub fn primary_ray_at(&self, pixel_x: u32, pixel_y: u32) -> Ray {
        let pixel_world_position = self.pixel00_world
            + (pixel_x as f64 * self.pixel_delta_u)
            + (pixel_y as f64 * self.pixel_delta_v);

        let direction = (pixel_world_position - self.camera_origin).normalized();
        Ray::new(self.camera_origin, direction)
    }
}

#[cfg(test)]
mod tests {
    use crate::math::Point3;
    use crate::scene::Camera;

    use super::PrimaryRayCamera;

    const EPSILON: f64 = 1.0e-12;

    fn assert_nearly_equal(lhs: f64, rhs: f64) {
        assert!(
            (lhs - rhs).abs() < EPSILON,
            "expected values to be nearly equal: lhs={lhs}, rhs={rhs}"
        );
    }

    #[test]
    fn center_pixel_points_toward_look_at_for_odd_resolution() {
        let camera = Camera::default();
        let ray_camera = PrimaryRayCamera::from_scene_camera(&camera, 3, 3);
        let center_ray = ray_camera.primary_ray_at(1, 1);

        assert_nearly_equal(center_ray.direction.x, 0.0);
        assert_nearly_equal(center_ray.direction.y, 0.0);
        assert_nearly_equal(center_ray.direction.z, -1.0);
    }

    #[test]
    fn top_and_bottom_rows_have_opposite_vertical_direction_signs() {
        let camera = Camera::default();
        let ray_camera = PrimaryRayCamera::from_scene_camera(&camera, 3, 3);
        let top_left_ray = ray_camera.primary_ray_at(0, 0);
        let bottom_left_ray = ray_camera.primary_ray_at(0, 2);

        assert!(top_left_ray.direction.y > 0.0);
        assert!(bottom_left_ray.direction.y < 0.0);
    }

    #[test]
    fn left_and_right_columns_have_opposite_horizontal_direction_signs() {
        let camera = Camera::default();
        let ray_camera = PrimaryRayCamera::from_scene_camera(&camera, 3, 3);
        let left_ray = ray_camera.primary_ray_at(0, 1);
        let right_ray = ray_camera.primary_ray_at(2, 1);

        assert!(left_ray.direction.x < 0.0);
        assert!(right_ray.direction.x > 0.0);
    }

    #[test]
    fn all_primary_rays_start_from_camera_origin() {
        let camera = Camera::default();
        let ray_camera = PrimaryRayCamera::from_scene_camera(&camera, 5, 7);

        for y in 0..7 {
            for x in 0..5 {
                let ray = ray_camera.primary_ray_at(x, y);
                assert_eq!(ray.origin, Point3::new(0.0, 0.0, 1.0));
            }
        }
    }
}
