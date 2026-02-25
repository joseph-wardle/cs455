use crate::math::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Lighting {
    pub direction_to_light: Vec3,
    pub light_color: Vec3,
    pub ambient_light: Vec3,
    pub background_color: Vec3,
}

impl Lighting {
    pub const fn new(
        direction_to_light: Vec3,
        light_color: Vec3,
        ambient_light: Vec3,
        background_color: Vec3,
    ) -> Self {
        Self {
            direction_to_light,
            light_color,
            ambient_light,
            background_color,
        }
    }
}

impl Default for Lighting {
    fn default() -> Self {
        Self::new(
            Vec3::new(0.0, 1.0, 0.0),
            Vec3::new(1.0, 1.0, 1.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::new(0.2, 0.2, 0.2),
        )
    }
}
