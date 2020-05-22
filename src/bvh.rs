use crate::geometry::*;
use rand::seq::SliceRandom;
use std::cmp::Ordering;
use std::ops::Range;
use std::sync::Arc;

pub struct BVHNode {
    aabb: AABB,
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
}

impl Hittable for BVHNode {
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
    pub fn from_hittables(mut objects: Vec<Box<dyn Hittable>>) -> Arc<BVHNode> {
        let comparator = &[compare_x, compare_y, compare_z]
            .choose(&mut rand::thread_rng())
            .unwrap();

        let num_objects = objects.len();

        let (left, right): (Arc<dyn Hittable>, Arc<dyn Hittable>) =
            match num_objects {
                1 => {
                    let arc: Arc<dyn Hittable> = objects.pop().unwrap().into();
                    (arc.clone(), arc.clone())
                }
                2 => {
                    let a1: Box<dyn Hittable> = objects.pop().unwrap();
                    let a2: Box<dyn Hittable> = objects.pop().unwrap();
                    match comparator(&a1, &a2) {
                        Ordering::Less => (a1.into(), a2.into()),
                        _ => (a2.into(), a1.into()),
                    }
                }
                _ => {
                    objects.sort_by(comparator);
                    let mid = num_objects / 2;
                    let obj_right = objects.split_off(mid);
                    let obj_left = objects;
                    (
                        BVHNode::from_hittables(obj_left),
                        BVHNode::from_hittables(obj_right),
                    )
                }
            };

        let aabb = AABB::containing(left.bounding_box(), right.bounding_box());
        Arc::new(BVHNode { left, right, aabb })
    }
}

fn compare_x(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
    let a_box = a.bounding_box();
    let b_box = b.bounding_box();
    a_box.cmp_axis(&b_box, 0)
}

fn compare_y(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
    let a_box = a.bounding_box();
    let b_box = b.bounding_box();
    a_box.cmp_axis(&b_box, 1)
}

fn compare_z(a: &Box<dyn Hittable>, b: &Box<dyn Hittable>) -> Ordering {
    let a_box = a.bounding_box();
    let b_box = b.bounding_box();
    a_box.cmp_axis(&b_box, 2)
}
