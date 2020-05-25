use super::{Hit, Hittable, Point, Ray, Vector, AABB};
use crate::material::Material;
use nalgebra::Unit;
use std::f64::consts::PI;
use std::ops::Range;
use std::sync::Arc;

pub struct Sphere {
    material: Arc<Material>,
    center: Point,
    radius: f64,
}

impl Sphere {
    pub fn new(
        material: Arc<Material>,
        center: Point,
        radius: f64,
    ) -> Arc<Sphere> {
        Arc::new(Sphere {
            material,
            center,
            radius,
        })
    }
}

impl Hittable for Sphere {
    fn is_light(&self) -> bool {
        match self.material.as_ref() {
            Material::Emission(..) => true,
            _ => false,
        }
    }

    fn bounding_box(&self) -> AABB {
        AABB::new(
            self.center - Vector::repeat(self.radius),
            self.center + Vector::repeat(self.radius),
        )
    }

    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        let oc = ray.origin - self.center;
        // Solve the quadratic formula.
        let (a, half_b, c) = (
            ray.dir.norm_squared(),
            oc.dot(&ray.dir),
            oc.norm_squared() - (self.radius * self.radius),
        );
        let discriminant = (half_b * half_b) - (a * c);
        if discriminant < 0.0 {
            None
        } else {
            // b^2 - 4ac > 0 ==> there is at least one root.
            // Subtract discriminant to find the smallest t such that
            // there's an intersection.
            let sqrt_disc = discriminant.sqrt();
            let t1: f64 = (-half_b - sqrt_disc) / a;
            let t2: f64 = (-half_b + sqrt_disc) / a;
            let t = if range.contains(&t1) {
                t1
            } else if range.contains(&t2) {
                t2
            } else {
                return None;
            };
            let point = ray.at(t);
            let normal = Unit::new_unchecked(
                (point - self.center) * self.radius.recip(),
            );
            Some(Hit::new(
                ray,
                normal,
                t,
                &self.material,
                self.get_uv(*normal),
            ))
        }
    }
}

impl Sphere {
    fn get_uv(&self, loc: Vector) -> (f64, f64) {
        let phi = loc.z.atan2(loc.x);
        let theta = loc.y.asin();
        let u = 1.0 - (phi + PI) / (2.0 * PI);
        let v = (theta + PI / 2.0) / PI;
        (u, v)
    }
}
