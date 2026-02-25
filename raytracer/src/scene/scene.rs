use super::{Camera, Lighting, Sphere};

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    camera: Camera,
    lighting: Lighting,
    spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new(camera: Camera, lighting: Lighting, spheres: Vec<Sphere>) -> Self {
        Self {
            camera,
            lighting,
            spheres,
        }
    }

    pub const fn camera(&self) -> &Camera {
        &self.camera
    }

    pub const fn lighting(&self) -> &Lighting {
        &self.lighting
    }

    pub fn spheres(&self) -> &[Sphere] {
        &self.spheres
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera::default(), Lighting::default(), Vec::new())
    }
}
