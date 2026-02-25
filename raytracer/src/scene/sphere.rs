use crate::math::Vec3;

use super::Material;

#[derive(Debug, Clone, PartialEq)]
pub struct Sphere {
    pub center: Vec3,
    pub radius: f64,
    pub material: Material,
}

impl Sphere {
    pub const fn new(center: Vec3, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius,
            material,
        }
    }
}
