use crate::geometry::*;

struct BVHNode {
    aabb: AABB,
    left: Box<dyn Hittable>,
    right: Box<dyn Hittable>,
}
