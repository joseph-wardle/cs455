use thiserror::Error;

use crate::math::Vec3;

const MIN_NONZERO_VECTOR_LENGTH: f64 = 1.0e-12;
const MIN_NONZERO_VECTOR_LENGTH_SQUARED: f64 =
    MIN_NONZERO_VECTOR_LENGTH * MIN_NONZERO_VECTOR_LENGTH;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SceneValidationError {
    #[error("field '{field}' must be finite, got {value}")]
    NonFiniteScalar { field: &'static str, value: f64 },

    #[error("field '{field}' must be in range [{min}, {max}], got {value}")]
    ScalarOutOfRange {
        field: &'static str,
        min: f64,
        max: f64,
        value: f64,
    },

    #[error("field '{field}' must be positive, got {value}")]
    NonPositiveScalar { field: &'static str, value: f64 },

    #[error("field '{field}' must contain finite components, got ({x}, {y}, {z})")]
    NonFiniteVector {
        field: &'static str,
        x: f64,
        y: f64,
        z: f64,
    },

    #[error("field '{field}' must be a non-zero vector")]
    ZeroLengthVector { field: &'static str },

    #[error("fields '{left_field}' and '{right_field}' must not be the same point")]
    CoincidentPoints {
        left_field: &'static str,
        right_field: &'static str,
    },
}

pub fn validate_finite_scalar(field: &'static str, value: f64) -> Result<(), SceneValidationError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SceneValidationError::NonFiniteScalar { field, value })
    }
}

pub fn validate_scalar_in_range(
    field: &'static str,
    value: f64,
    min: f64,
    max: f64,
) -> Result<(), SceneValidationError> {
    validate_finite_scalar(field, value)?;
    if value < min || value > max {
        Err(SceneValidationError::ScalarOutOfRange {
            field,
            min,
            max,
            value,
        })
    } else {
        Ok(())
    }
}

pub fn validate_positive_scalar(
    field: &'static str,
    value: f64,
) -> Result<(), SceneValidationError> {
    validate_finite_scalar(field, value)?;
    if value > 0.0 {
        Ok(())
    } else {
        Err(SceneValidationError::NonPositiveScalar { field, value })
    }
}

pub fn validate_finite_vector(
    field: &'static str,
    value: Vec3,
) -> Result<(), SceneValidationError> {
    if value.x.is_finite() && value.y.is_finite() && value.z.is_finite() {
        Ok(())
    } else {
        Err(SceneValidationError::NonFiniteVector {
            field,
            x: value.x,
            y: value.y,
            z: value.z,
        })
    }
}

pub fn validate_nonzero_vector(
    field: &'static str,
    value: Vec3,
) -> Result<(), SceneValidationError> {
    validate_finite_vector(field, value)?;
    if value.length_squared() > MIN_NONZERO_VECTOR_LENGTH_SQUARED {
        Ok(())
    } else {
        Err(SceneValidationError::ZeroLengthVector { field })
    }
}

pub fn validate_distinct_points(
    left_field: &'static str,
    left_point: Vec3,
    right_field: &'static str,
    right_point: Vec3,
) -> Result<(), SceneValidationError> {
    validate_finite_vector(left_field, left_point)?;
    validate_finite_vector(right_field, right_point)?;

    if (left_point - right_point).length_squared() > MIN_NONZERO_VECTOR_LENGTH_SQUARED {
        Ok(())
    } else {
        Err(SceneValidationError::CoincidentPoints {
            left_field,
            right_field,
        })
    }
}

pub fn validate_color_rgb(field: &'static str, color: Vec3) -> Result<(), SceneValidationError> {
    validate_finite_vector(field, color)?;
    validate_scalar_in_range(field, color.x, 0.0, 1.0)?;
    validate_scalar_in_range(field, color.y, 0.0, 1.0)?;
    validate_scalar_in_range(field, color.z, 0.0, 1.0)?;
    Ok(())
}
