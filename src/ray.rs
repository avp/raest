use crate::geometry::*;

#[derive(Debug, Copy, Clone)]
pub struct Ray {
    pub origin: Point,
    pub dir: Vector,
}
