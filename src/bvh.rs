use crate::geometry::*;
use std::ops::Range;

struct BVHNode {
    aabb: AABB,
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
}

impl Hittable for BVHNode {
    fn bounding_box(&self) -> AABB {
        self.aabb
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        if !self.aabb.hit(ray, range) {
            return None;
        }
        let left = self.left.hit(ray, range);
        let start = match left {
            None => range.start,
            Some(hit) => hit.t,
        };
        self.right.hit(ray, start..range.end).or(left)
    }
}
