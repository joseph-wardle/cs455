use super::{Camera, Lighting, Sphere, Triangle};

#[derive(Debug, Clone, PartialEq)]
pub struct Scene {
    camera: Camera,
    lighting: Lighting,
    spheres: Vec<Sphere>,
    triangles: Vec<Triangle>,
}

impl Scene {
    pub fn new(
        camera: Camera,
        lighting: Lighting,
        spheres: Vec<Sphere>,
        triangles: Vec<Triangle>,
    ) -> Self {
        Self {
            camera,
            lighting,
            spheres,
            triangles,
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

    pub fn triangles(&self) -> &[Triangle] {
        &self.triangles
    }
}

impl Default for Scene {
    fn default() -> Self {
        Self::new(
            Camera::default(),
            Lighting::default(),
            Vec::new(),
            Vec::new(),
        )
    }
}
