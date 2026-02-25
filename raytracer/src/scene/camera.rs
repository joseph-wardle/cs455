use crate::math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Camera {
    pub look_from: Vec3,
    pub look_at: Vec3,
    pub look_up: Vec3,
    pub field_of_view_degrees: f64,
}

impl Camera {
    pub const fn new(
        look_from: Vec3,
        look_at: Vec3,
        look_up: Vec3,
        field_of_view_degrees: f64,
    ) -> Self {
        Self {
            look_from,
            look_at,
            look_up,
            field_of_view_degrees,
        }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(
            Vec3::new(0.0, 0.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            90.0,
        )
    }
}
