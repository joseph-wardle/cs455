use crate::scene::Scene;

use super::RenderImage;

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
        let background_rgb8 = scene.lighting.background_color.to_rgb8();
        RenderImage::new_solid(
            self.config.image_width,
            self.config.image_height,
            background_rgb8,
        )
    }
}
