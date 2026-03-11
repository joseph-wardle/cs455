use crate::math::Point3;

use super::validation::{validate_distinct_points, validate_nonzero_vector};
use super::{Material, SceneValidationError};

#[derive(Debug, Clone, PartialEq)]
pub struct Triangle {
    vertex0: Point3,
    vertex1: Point3,
    vertex2: Point3,
    material: Material,
}

impl Triangle {
    pub fn try_new(
        vertex0: Point3,
        vertex1: Point3,
        vertex2: Point3,
        material: Material,
    ) -> Result<Self, SceneValidationError> {
        validate_distinct_points("V0", vertex0, "V1", vertex1)?;
        validate_distinct_points("V1", vertex1, "V2", vertex2)?;
        validate_distinct_points("V2", vertex2, "V0", vertex0)?;

        let edge01 = vertex1 - vertex0;
        let edge02 = vertex2 - vertex0;
        validate_nonzero_vector("Triangle", edge01.cross(edge02))?;

        Ok(Self {
            vertex0,
            vertex1,
            vertex2,
            material,
        })
    }

    pub const fn vertex0(&self) -> Point3 {
        self.vertex0
    }

    pub const fn vertex1(&self) -> Point3 {
        self.vertex1
    }

    pub const fn vertex2(&self) -> Point3 {
        self.vertex2
    }

    pub const fn material(&self) -> &Material {
        &self.material
    }
}

#[cfg(test)]
mod tests {
    use crate::math::{ColorRgb, Point3};

    use super::*;

    fn test_material() -> Material {
        Material::try_new(
            0.8,
            0.1,
            0.1,
            ColorRgb::new(1.0, 1.0, 1.0),
            ColorRgb::new(1.0, 1.0, 1.0),
            16.0,
            0.0,
        )
        .expect("test material values should be valid")
    }

    #[test]
    fn rejects_triangle_with_coincident_vertices() {
        let result = Triangle::try_new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(0.0, 1.0, 0.0),
            test_material(),
        );

        assert!(matches!(
            result,
            Err(SceneValidationError::CoincidentPoints { .. })
        ));
    }

    #[test]
    fn rejects_triangle_with_collinear_vertices() {
        let result = Triangle::try_new(
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(1.0, 0.0, 0.0),
            Point3::new(2.0, 0.0, 0.0),
            test_material(),
        );

        assert!(matches!(
            result,
            Err(SceneValidationError::ZeroLengthVector { field: "Triangle" })
        ));
    }
}
