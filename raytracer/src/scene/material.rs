use crate::math::ColorRgb;

use super::SceneValidationError;
use super::validation::{validate_color_rgb, validate_positive_scalar, validate_scalar_in_range};

#[derive(Debug, Clone, PartialEq)]
pub struct Material {
    diffuse_weight: f64,
    specular_weight: f64,
    ambient_weight: f64,
    diffuse_color: ColorRgb,
    specular_color: ColorRgb,
    gloss_exponent: f64,
}

impl Material {
    pub fn try_new(
        diffuse_weight: f64,
        specular_weight: f64,
        ambient_weight: f64,
        diffuse_color: ColorRgb,
        specular_color: ColorRgb,
        gloss_exponent: f64,
    ) -> Result<Self, SceneValidationError> {
        validate_scalar_in_range("Kd", diffuse_weight, 0.0, 1.0)?;
        validate_scalar_in_range("Ks", specular_weight, 0.0, 1.0)?;
        validate_scalar_in_range("Ka", ambient_weight, 0.0, 1.0)?;
        validate_color_rgb("Od", diffuse_color)?;
        validate_color_rgb("Os", specular_color)?;
        validate_positive_scalar("Kgls", gloss_exponent)?;

        Ok(Self {
            diffuse_weight,
            specular_weight,
            ambient_weight,
            diffuse_color,
            specular_color,
            gloss_exponent,
        })
    }

    pub const fn diffuse_weight(&self) -> f64 {
        self.diffuse_weight
    }

    pub const fn specular_weight(&self) -> f64 {
        self.specular_weight
    }

    pub const fn ambient_weight(&self) -> f64 {
        self.ambient_weight
    }

    pub const fn diffuse_color(&self) -> ColorRgb {
        self.diffuse_color
    }

    pub const fn specular_color(&self) -> ColorRgb {
        self.specular_color
    }

    pub const fn gloss_exponent(&self) -> f64 {
        self.gloss_exponent
    }
}

impl Default for Material {
    fn default() -> Self {
        Self::try_new(
            0.7,
            0.2,
            0.1,
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
        )
        .expect("default material values must be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_weights_outside_unit_interval() {
        let result = Material::try_new(
            1.1,
            0.2,
            0.1,
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
        );

        assert!(matches!(
            result,
            Err(SceneValidationError::ScalarOutOfRange { .. })
        ));
    }
}
