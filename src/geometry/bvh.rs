use super::{Hit, Hittable, Ray, AABB};
use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

pub struct BVHNode {
    aabb: AABB,
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
}

impl Hittable for BVHNode {
    fn is_light(&self) -> bool {
        false
    }

    fn bounding_box(&self) -> AABB {
        self.aabb
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        if !self.aabb.hit(ray, range.clone()) {
            return None;
        }
        let left = self.left.hit(ray, range.clone());
        let end = match &left {
            None => range.end,
            Some(hit) => hit.t,
        };
        self.right.hit(ray, range.start..end).or(left)
    }
}

impl BVHNode {
    pub fn from_hittables(objects: Vec<Arc<dyn Hittable>>) -> Arc<BVHNode> {
        Self::from_hittables_along_axis(objects, 0)
    }

    fn from_hittables_along_axis(
        mut objects: Vec<Arc<dyn Hittable>>,
        axis: usize,
    ) -> Arc<BVHNode> {
        assert!(!objects.is_empty(), "BVH must contain at least one object");
        let comparator = |a: &dyn Hittable, b: &dyn Hittable| -> Ordering {
            let a_box = a.bounding_box();
            let b_box = b.bounding_box();
            a_box.cmp_axis(&b_box, axis)
        };

        let num_objects = objects.len();

        let (left, right): (Arc<dyn Hittable>, Arc<dyn Hittable>) =
            match num_objects {
                1 => {
                    let arc: Arc<dyn Hittable> = objects.pop().unwrap().into();
                    (arc.clone(), arc.clone())
                }
                2 => {
                    let a1: Arc<dyn Hittable> = objects.pop().unwrap();
                    let a2: Arc<dyn Hittable> = objects.pop().unwrap();
                    match comparator(&*a1, &*a2) {
                        Ordering::Less => (a1.into(), a2.into()),
                        _ => (a2.into(), a1.into()),
                    }
                }
                _ => {
                    objects.sort_by(|a, b| comparator(&**a, &**b));
                    let mid = num_objects / 2;
                    let obj_right = objects.split_off(mid);
                    let obj_left = objects;
                    (
                        BVHNode::from_hittables_along_axis(
                            obj_left,
                            (axis + 1) % 3,
                        ),
                        BVHNode::from_hittables_along_axis(
                            obj_right,
                            (axis + 1) % 3,
                        ),
                    )
                }
            };

        let aabb = AABB::containing(left.bounding_box(), right.bounding_box());
        Arc::new(BVHNode { left, right, aabb })
    }
}
