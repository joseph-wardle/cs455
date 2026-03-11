use super::{Direction3, Point3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Direction3,
}

impl Ray {
    pub const fn new(origin: Point3, direction: Direction3) -> Self {
        Self { origin, direction }
    }

    pub fn at(self, distance: f64) -> Point3 {
        self.origin + self.direction * distance
    }
}
