use crate::math::{ColorRgb, Direction3, Ray};
use crate::scene::{Material, Scene};

use super::{
    HitObject, PrimaryRayCamera, RenderImage, SurfaceHit, closest_scene_hit,
    shade_hit_with_directional_light,
};

const RAY_T_MIN: f64 = 1.0e-6;
const REFLECTION_RAY_BIAS: f64 = 1.0e-4;
const DEFAULT_MAX_REFLECTION_DEPTH: u32 = 4;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderConfig {
    pub image_width: u32,
    pub image_height: u32,
    pub max_reflection_depth: u32,
}

impl RenderConfig {
    pub const fn new(image_width: u32, image_height: u32) -> Self {
        Self::with_max_reflection_depth(image_width, image_height, DEFAULT_MAX_REFLECTION_DEPTH)
    }

    pub const fn with_max_reflection_depth(
        image_width: u32,
        image_height: u32,
        max_reflection_depth: u32,
    ) -> Self {
        Self {
            image_width,
            image_height,
            max_reflection_depth,
        }
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self::new(500, 500)
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Renderer {
    config: RenderConfig,
}

impl Renderer {
    pub const fn new(config: RenderConfig) -> Self {
        Self { config }
    }

    pub fn render(&self, scene: &Scene) -> RenderImage {
        let background_rgb8 = scene.lighting().background_color().to_rgb8();
        let normalized_direction_to_light = scene.lighting().direction_to_light().normalized();
        let mut image = RenderImage::new_solid(
            self.config.image_width,
            self.config.image_height,
            background_rgb8,
        );
        let ray_camera = PrimaryRayCamera::from_scene_camera(
            scene.camera(),
            self.config.image_width,
            self.config.image_height,
        );

        for pixel_y in 0..self.config.image_height {
            for pixel_x in 0..self.config.image_width {
                let primary_ray = ray_camera.primary_ray_at(pixel_x, pixel_y);
                let traced_color = self.trace_ray(
                    scene,
                    primary_ray,
                    self.config.max_reflection_depth,
                    normalized_direction_to_light,
                );
                image.set_pixel_rgb8(pixel_x, pixel_y, traced_color.to_rgb8());
            }
        }

        image
    }

    fn trace_ray(
        &self,
        scene: &Scene,
        ray: Ray,
        depth_remaining: u32,
        normalized_direction_to_light: Direction3,
    ) -> ColorRgb {
        let Some(hit) = closest_scene_hit(
            ray,
            scene.spheres(),
            scene.triangles(),
            RAY_T_MIN,
            f64::INFINITY,
        ) else {
            return scene.lighting().background_color();
        };

        let local_color = shade_hit_with_directional_light(
            scene,
            hit,
            normalized_direction_to_light,
            ray.direction,
        );
        let material = material_for_hit(scene, hit);
        let reflection_weight = material.reflection_weight();
        if reflection_weight <= 0.0 || depth_remaining == 0 {
            return local_color;
        }

        let reflection_ray = spawn_reflection_ray(hit, ray.direction);
        let reflected_color = self.trace_ray(
            scene,
            reflection_ray,
            depth_remaining - 1,
            normalized_direction_to_light,
        );

        (local_color * (1.0 - reflection_weight)) + (reflected_color * reflection_weight)
    }
}

fn material_for_hit<'scene>(scene: &'scene Scene, hit: SurfaceHit) -> &'scene Material {
    match hit.hit_object {
        HitObject::Sphere(sphere_index) => scene.spheres()[sphere_index].material(),
        HitObject::Triangle(triangle_index) => scene.triangles()[triangle_index].material(),
    }
}

fn spawn_reflection_ray(hit: SurfaceHit, incoming_ray_direction: Direction3) -> Ray {
    let unit_normal = hit.outward_normal.normalized();
    let reflected_direction = reflect_direction(incoming_ray_direction, unit_normal).normalized();
    let offset_normal = if reflected_direction.dot(unit_normal) >= 0.0 {
        unit_normal
    } else {
        -unit_normal
    };

    let origin = hit.point + (REFLECTION_RAY_BIAS * offset_normal);
    Ray::new(origin, reflected_direction)
}

fn reflect_direction(incident_direction: Direction3, unit_normal: Direction3) -> Direction3 {
    incident_direction - (2.0 * incident_direction.dot(unit_normal) * unit_normal)
}

#[cfg(test)]
mod tests {
    use crate::math::{ColorRgb, Direction3, Point3};
    use crate::scene::{Camera, Lighting, Material, Scene, Sphere};

    use super::{RenderConfig, Renderer};

    #[test]
    fn render_without_spheres_is_background_only() {
        let scene = Scene::new(
            Camera::default(),
            Lighting::default(),
            Vec::new(),
            Vec::new(),
        );
        let renderer = Renderer::new(RenderConfig::new(3, 3));

        let image = renderer.render(&scene);
        let expected_background = scene.lighting().background_color().to_rgb8();

        for pixel in image.as_rgb8().chunks_exact(3) {
            assert_eq!(pixel, expected_background);
        }
    }

    #[test]
    fn center_pixel_uses_hit_sphere_diffuse_color() {
        let material = Material::try_new(
            1.0,
            0.0,
            0.0,
            ColorRgb::new(1.0, 0.0, 0.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
            0.0,
        )
        .expect("test material should be valid");
        let lighting = Lighting::try_new(
            Direction3::new(0.0, 0.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::zero(),
            ColorRgb::new(0.2, 0.2, 0.2),
        )
        .expect("test lighting should be valid");
        let sphere = Sphere::try_new(Point3::new(0.0, 0.0, 0.0), 0.4, material)
            .expect("test sphere should be valid");
        let scene = Scene::new(Camera::default(), lighting, vec![sphere], Vec::new());
        let renderer = Renderer::new(RenderConfig::new(3, 3));

        let image = renderer.render(&scene);
        assert_eq!(image.pixel_rgb8(1, 1), [255, 0, 0]);
    }

    #[test]
    fn center_pixel_in_shadow_keeps_only_ambient_component() {
        let lit_sphere_material = Material::try_new(
            1.0,
            0.0,
            1.0,
            ColorRgb::new(1.0, 0.0, 0.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
            0.0,
        )
        .expect("test material should be valid");
        let occluder_material = Material::default();
        let lighting = Lighting::try_new(
            Direction3::new(0.0, 1.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(0.1, 0.1, 0.1),
            ColorRgb::new(0.2, 0.2, 0.2),
        )
        .expect("test lighting should be valid");

        let lit_sphere = Sphere::try_new(Point3::new(0.0, 0.0, 0.0), 0.4, lit_sphere_material)
            .expect("test sphere should be valid");
        let occluder = Sphere::try_new(Point3::new(0.0, 0.25, 0.65), 0.1, occluder_material)
            .expect("test sphere should be valid");
        let scene = Scene::new(
            Camera::default(),
            lighting,
            vec![lit_sphere, occluder],
            Vec::new(),
        );
        let renderer = Renderer::new(RenderConfig::new(3, 3));

        let image = renderer.render(&scene);
        assert_eq!(image.pixel_rgb8(1, 1), [26, 0, 0]);
    }

    #[test]
    fn center_pixel_receives_specular_highlight() {
        let material = Material::try_new(
            0.0,
            1.0,
            0.0,
            ColorRgb::zero(),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
            0.0,
        )
        .expect("test material should be valid");
        let lighting = Lighting::try_new(
            Direction3::new(0.0, 0.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::zero(),
            ColorRgb::new(0.2, 0.2, 0.2),
        )
        .expect("test lighting should be valid");
        let sphere = Sphere::try_new(Point3::new(0.0, 0.0, 0.0), 0.4, material)
            .expect("test sphere should be valid");
        let scene = Scene::new(Camera::default(), lighting, vec![sphere], Vec::new());
        let renderer = Renderer::new(RenderConfig::new(3, 3));

        let image = renderer.render(&scene);
        assert_eq!(image.pixel_rgb8(1, 1), [255, 255, 255]);
    }

    #[test]
    fn center_pixel_reflects_background_when_reflection_depth_is_available() {
        let reflective_material = Material::try_new(
            0.0,
            0.0,
            0.0,
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
            1.0,
        )
        .expect("test material should be valid");
        let lighting = Lighting::try_new(
            Direction3::new(0.0, 1.0, 0.0),
            ColorRgb::zero(),
            ColorRgb::zero(),
            ColorRgb::new(0.2, 0.2, 0.2),
        )
        .expect("test lighting should be valid");
        let reflective_sphere =
            Sphere::try_new(Point3::new(0.0, 0.0, 0.0), 0.4, reflective_material)
                .expect("test sphere should be valid");
        let scene = Scene::new(
            Camera::default(),
            lighting,
            vec![reflective_sphere],
            Vec::new(),
        );
        let renderer = Renderer::new(RenderConfig::with_max_reflection_depth(3, 3, 1));

        let image = renderer.render(&scene);
        assert_eq!(image.pixel_rgb8(1, 1), [51, 51, 51]);
    }

    #[test]
    fn center_pixel_uses_only_local_shading_when_reflection_depth_is_zero() {
        let reflective_material = Material::try_new(
            0.0,
            0.0,
            0.0,
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
            1.0,
        )
        .expect("test material should be valid");
        let lighting = Lighting::try_new(
            Direction3::new(0.0, 1.0, 0.0),
            ColorRgb::zero(),
            ColorRgb::zero(),
            ColorRgb::new(0.2, 0.2, 0.2),
        )
        .expect("test lighting should be valid");
        let reflective_sphere =
            Sphere::try_new(Point3::new(0.0, 0.0, 0.0), 0.4, reflective_material)
                .expect("test sphere should be valid");
        let scene = Scene::new(
            Camera::default(),
            lighting,
            vec![reflective_sphere],
            Vec::new(),
        );
        let renderer = Renderer::new(RenderConfig::with_max_reflection_depth(3, 3, 0));

        let image = renderer.render(&scene);
        assert_eq!(image.pixel_rgb8(1, 1), [0, 0, 0]);
    }
}
