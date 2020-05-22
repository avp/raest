use crate::geometry::*;
use std::cmp::Ordering;
use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub(super) struct AABB {
    pub min: Point,
    pub max: Point,
}

impl AABB {
    pub fn new(min: Point, max: Point) -> AABB {
        AABB { min, max }
    }

    pub fn containing(box1: AABB, box2: AABB) -> AABB {
        let min = Point::new(
            f64::min(box1.min.x, box2.min.x),
            f64::min(box1.min.y, box2.min.y),
            f64::min(box1.min.z, box2.min.z),
        );
        let max = Point::new(
            f64::max(box1.max.x, box2.max.x),
            f64::max(box1.max.y, box2.max.y),
            f64::max(box1.max.z, box2.max.z),
        );
        AABB::new(min, max)
    }

    pub fn hit(&self, ray: Ray, mut range: Range<f64>) -> bool {
        for dim in 0..self.min.len() {
            let t0 = (self.min[dim] - ray.origin[dim]) * (1.0 / ray.dir[dim]);
            let t1 = (self.max[dim] - ray.origin[dim]) * (1.0 / ray.dir[dim]);
            let (t_in, t_out) = if t0 < t1 { (t0, t1) } else { (t1, t0) };
            range.start = f64::max(range.start, t_in);
            range.end = f64::min(range.end, t_out);
            if range.end <= range.start {
                return false;
            }
        }
        true
    }

    pub fn cmp_axis(&self, other: &AABB, axis: usize) -> Ordering {
        self.min[axis].partial_cmp(&other.min[axis]).unwrap()
    }
}
