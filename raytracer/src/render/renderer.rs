use crate::scene::Scene;

use super::{PrimaryRayCamera, RenderImage, closest_sphere_hit, shade_hit_with_directional_light};

const PRIMARY_RAY_T_MIN: f64 = 1.0e-6;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderConfig {
    pub image_width: u32,
    pub image_height: u32,
}

impl RenderConfig {
    pub const fn new(image_width: u32, image_height: u32) -> Self {
        Self {
            image_width,
            image_height,
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
        let spheres = scene.spheres();
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
                let hit =
                    closest_sphere_hit(primary_ray, spheres, PRIMARY_RAY_T_MIN, f64::INFINITY);

                if let Some(hit) = hit {
                    let shaded_color =
                        shade_hit_with_directional_light(scene, hit, normalized_direction_to_light);
                    image.set_pixel_rgb8(pixel_x, pixel_y, shaded_color.to_rgb8());
                }
            }
        }

        image
    }
}

#[cfg(test)]
mod tests {
    use crate::math::{ColorRgb, Direction3, Point3};
    use crate::scene::{Camera, Lighting, Material, Scene, Sphere};

    use super::{RenderConfig, Renderer};

    #[test]
    fn render_without_spheres_is_background_only() {
        let scene = Scene::new(Camera::default(), Lighting::default(), Vec::new());
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
        let scene = Scene::new(Camera::default(), lighting, vec![sphere]);
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
        let scene = Scene::new(Camera::default(), lighting, vec![lit_sphere, occluder]);
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
        let scene = Scene::new(Camera::default(), lighting, vec![sphere]);
        let renderer = Renderer::new(RenderConfig::new(3, 3));

        let image = renderer.render(&scene);
        assert_eq!(image.pixel_rgb8(1, 1), [255, 255, 255]);
    }
}
