use crate::math::Point3;

use super::validation::{validate_finite_vector, validate_positive_scalar};
use super::{Material, SceneValidationError};

#[derive(Debug, Clone, PartialEq)]
pub struct Sphere {
    center: Point3,
    radius: f64,
    material: Material,
}

impl Sphere {
    pub fn try_new(
        center: Point3,
        radius: f64,
        material: Material,
    ) -> Result<Self, SceneValidationError> {
        validate_finite_vector("Center", center)?;
        validate_positive_scalar("Radius", radius)?;

        Ok(Self {
            center,
            radius,
            material,
        })
    }

    pub const fn center(&self) -> Point3 {
        self.center
    }

    pub const fn radius(&self) -> f64 {
        self.radius
    }

    pub const fn material(&self) -> &Material {
        &self.material
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_positive_radius() {
        let result = Sphere::try_new(Point3::zero(), 0.0, Material::default());
        assert!(matches!(
            result,
            Err(SceneValidationError::NonPositiveScalar { .. })
        ));
    }
}
