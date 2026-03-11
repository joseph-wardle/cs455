use crate::math::{Direction3, Point3};

use super::SceneValidationError;
use super::validation::{
    validate_distinct_points, validate_nonzero_vector, validate_scalar_in_range,
};

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    look_from: Point3,
    look_at: Point3,
    look_up: Direction3,
    field_of_view_degrees: f64,
}

impl Camera {
    pub fn try_new(
        look_from: Point3,
        look_at: Point3,
        look_up: Direction3,
        field_of_view_degrees: f64,
    ) -> Result<Self, SceneValidationError> {
        validate_distinct_points("CameraLookFrom", look_from, "CameraLookAt", look_at)?;
        validate_nonzero_vector("CameraLookUp", look_up)?;
        validate_scalar_in_range("FieldOfView", field_of_view_degrees, 1.0, 179.0)?;

        Ok(Self {
            look_from,
            look_at,
            look_up,
            field_of_view_degrees,
        })
    }

    pub const fn look_from(&self) -> Point3 {
        self.look_from
    }

    pub const fn look_at(&self) -> Point3 {
        self.look_at
    }

    pub const fn look_up(&self) -> Direction3 {
        self.look_up
    }

    pub const fn field_of_view_degrees(&self) -> f64 {
        self.field_of_view_degrees
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::try_new(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 1.0, 0.0),
            90.0,
        )
        .expect("default camera values must be valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_field_of_view() {
        let result = Camera::try_new(
            Point3::new(0.0, 0.0, 1.0),
            Point3::new(0.0, 0.0, 0.0),
            Direction3::new(0.0, 1.0, 0.0),
            0.0,
        );

        assert!(matches!(
            result,
            Err(SceneValidationError::ScalarOutOfRange { .. })
        ));
    }
}
