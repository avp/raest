use crate::geometry::*;
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct AABB {
    a: Point,
    b: Point,
}

impl AABB {
    pub fn new(a: Point, b: Point) -> AABB {
        AABB { a, b }
    }

    pub fn hit(&self, ray: Ray, mut range: Range<f64>) -> bool {
        for dim in 0..self.a.len() {
            let t0 = (self.a[dim] - ray.origin[dim]) * (1.0 / ray.dir[dim]);
            let t1 = (self.b[dim] - ray.origin[dim]) * (1.0 / ray.dir[dim]);
            let t_in = f64::min(t0, t1);
            let t_out = f64::max(t0, t1);
            range.start = f64::max(range.start, t_in);
            range.end = f64::min(range.end, t_out);
            if range.end <= range.start {
                return false;
            }
        }
        true
    }
}
