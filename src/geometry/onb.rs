use super::*;
use nalgebra::Unit;

/// Represents an orthonormal basis of vectors.
#[derive(Debug, Copy, Clone)]
pub struct ONB {
    pub u: Unit<Vector>,
    pub v: Unit<Vector>,
    pub w: Unit<Vector>,
}

impl ONB {
    pub fn from_w(w: Unit<Vector>) -> ONB {
        let a = if w.x.abs() > 0.9 {
            Vector::y_axis()
        } else {
            Vector::x_axis()
        };
        let v = Unit::new_unchecked(w.cross(&a));
        let u = Unit::new_unchecked(w.cross(&v));
        ONB { u, v, w }
    }

    pub fn localize(&self, xyz: Vector) -> Vector {
        (xyz.x * *self.u) + (xyz.y * *self.v) + (xyz.z * *self.w)
    }
}
