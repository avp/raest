use crate::geometry::*;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Point,
    pub dir: Vector,
}

impl Ray {
    pub fn at(&self, t: f64) -> Point {
        self.origin + t * self.dir
    }
}
