use super::{Camera, Lighting, Sphere};

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    pub camera: Camera,
    pub lighting: Lighting,
    pub spheres: Vec<Sphere>,
}

impl Scene {
    pub fn new(camera: Camera, lighting: Lighting, spheres: Vec<Sphere>) -> Self {
        Self {
            camera,
            lighting,
            spheres,
        }
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(Camera::default(), Lighting::default(), Vec::new())
    }
}
