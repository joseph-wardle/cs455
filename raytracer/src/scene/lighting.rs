use crate::math::{ColorRgb, Direction3};

use super::SceneValidationError;
use super::validation::{validate_color_rgb, validate_nonzero_vector};

#[derive(Debug, Clone, PartialEq)]
pub struct Lighting {
    direction_to_light: Direction3,
    light_color: ColorRgb,
    ambient_light: ColorRgb,
    background_color: ColorRgb,
}

impl Lighting {
    pub fn try_new(
        direction_to_light: Direction3,
        light_color: ColorRgb,
        ambient_light: ColorRgb,
        background_color: ColorRgb,
    ) -> Result<Self, SceneValidationError> {
        validate_nonzero_vector("DirectionToLight", direction_to_light)?;
        validate_color_rgb("LightColor", light_color)?;
        validate_color_rgb("AmbientLight", ambient_light)?;
        validate_color_rgb("BackgroundColor", background_color)?;

        Ok(Self {
            direction_to_light,
            light_color,
            ambient_light,
            background_color,
        })
    }

    pub const fn direction_to_light(&self) -> Direction3 {
        self.direction_to_light
    }

    pub const fn light_color(&self) -> ColorRgb {
        self.light_color
    }

    pub const fn ambient_light(&self) -> ColorRgb {
        self.ambient_light
    }

    pub const fn background_color(&self) -> ColorRgb {
        self.background_color
    }
}

impl Default for Lighting {
    fn default() -> Self {
        Self::try_new(
            Direction3::new(0.0, 1.0, 0.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(0.0, 0.0, 0.0),
            ColorRgb::new(0.2, 0.2, 0.2),
        )
        .expect("default lighting values must be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_zero_light_direction() {
        let result = Lighting::try_new(
            Direction3::zero(),
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(0.0, 0.0, 0.0),
            ColorRgb::new(0.2, 0.2, 0.2),
        );

        assert!(matches!(
            result,
            Err(SceneValidationError::ZeroLengthVector { .. })
        ));
    }
}
