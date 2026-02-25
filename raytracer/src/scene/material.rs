use crate::math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    pub diffuse_weight: f64,
    pub specular_weight: f64,
    pub ambient_weight: f64,
    pub diffuse_color: Vec3,
    pub specular_color: Vec3,
    pub gloss_exponent: f64,
}

impl Material {
    pub const fn new(
        diffuse_weight: f64,
        specular_weight: f64,
        ambient_weight: f64,
        diffuse_color: Vec3,
        specular_color: Vec3,
        gloss_exponent: f64,
    ) -> Self {
        Self {
            diffuse_weight,
            specular_weight,
            ambient_weight,
            diffuse_color,
            specular_color,
            gloss_exponent,
        }
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::new(
            0.7,
            0.2,
            0.1,
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(1.0, 1.0, 1.0),
            16.0,
        )
    }
}
