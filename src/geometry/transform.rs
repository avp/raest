use super::*;
use nalgebra::{Rotation3, UnitQuaternion};
use std::ops::Range;

pub struct Translate {
    target: Arc<dyn Hittable>,
    offset: Vector,
}

impl Translate {
    pub(super) fn new(
        target: Arc<dyn Hittable>,
        offset: Vector,
    ) -> Arc<Translate> {
        Arc::new(Translate { target, offset })
    }
}

impl Hittable for Translate {
    fn is_light(&self) -> bool {
        self.target.is_light()
    }

    fn bounding_box(&self) -> AABB {
        let aabb = self.target.bounding_box();
        AABB::new(aabb.min + self.offset, aabb.max + self.offset)
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        let moved_ray = Ray {
            origin: ray.origin - self.offset,
            dir: ray.dir,
        };
        match self.target.hit(moved_ray, range) {
            None => None,
            Some(hit) => {
                Some(Hit::new(ray, hit.normal, hit.t, hit.material, hit.uv))
            }
        }
    }
}

pub struct Rotate {
    target: Arc<dyn Hittable>,
    offset: UnitQuaternion<f64>,
}

impl Rotate {
    pub(super) fn new(
        target: Arc<dyn Hittable>,
        offset: Rotation3<f64>,
    ) -> Arc<Rotate> {
        Arc::new(Rotate {
            target,
            offset: UnitQuaternion::from_rotation_matrix(&offset),
        })
    }
}

impl Hittable for Rotate {
    fn is_light(&self) -> bool {
        self.target.is_light()
    }

    fn bounding_box(&self) -> AABB {
        let aabb = self.target.bounding_box();
        let points = [
            Point::new(aabb.min.x, aabb.min.y, aabb.min.z),
            Point::new(aabb.min.x, aabb.min.y, aabb.max.z),
            Point::new(aabb.min.x, aabb.max.y, aabb.min.z),
            Point::new(aabb.min.x, aabb.max.y, aabb.max.z),
            Point::new(aabb.max.x, aabb.min.y, aabb.min.z),
            Point::new(aabb.max.x, aabb.min.y, aabb.max.z),
            Point::new(aabb.max.x, aabb.max.y, aabb.min.z),
            Point::new(aabb.max.x, aabb.max.y, aabb.max.z),
        ];
        let mut min = Point::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max =
            Point::new(-f64::INFINITY, -f64::INFINITY, -f64::INFINITY);
        for p in &points {
            let tp = self.offset.transform_point(&p);
            for c in 0..3 {
                if tp[c] < min[c] {
                    min[c] = tp[c];
                }
                if tp[c] > max[c] {
                    max[c] = tp[c];
                }
            }
        }
        AABB::new(min, max)
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        let rotated_ray = Ray {
            origin: self.offset.inverse_transform_point(&ray.origin),
            dir: self.offset.inverse_transform_vector(&ray.dir),
        };
        match self.target.hit(rotated_ray, range) {
            None => None,
            Some(hit) => Some(Hit::new(
                ray,
                Unit::new_unchecked(self.offset.transform_vector(&*hit.normal)),
                hit.t,
                hit.material,
                hit.uv,
            )),
        }
    }
}
