use crate::geometry::*;
use std::cmp::Ordering;
use std::ops::Range;

#[derive(Debug, Copy, Clone)]
pub struct AABB {
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

    pub fn hit(&self, ray: Ray, range: Range<f64>) -> bool {
        let t0s = (self.min - ray.origin).component_div(&ray.dir);
        let t1s = (self.max - ray.origin).component_div(&ray.dir);
        let (t_in, t_out) = t0s.inf_sup(&t1s);
        let start = f64::max(range.start, t_in.max());
        let end = f64::min(range.end, t_out.min());
        end > start
    }

    pub fn cmp_axis(&self, other: &AABB, axis: usize) -> Ordering {
        self.min[axis].partial_cmp(&other.min[axis]).unwrap()
    }
}
