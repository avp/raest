use nalgebra::{Point3, Vector3};
use std::ops::{Bound, RangeBounds};

pub use crate::material::Material;
pub use crate::ray::Ray;

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;

#[derive(Debug)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
}

fn bound_cloned<T: Clone>(b: Bound<&T>) -> Bound<T> {
    match b {
        Bound::Unbounded => Bound::Unbounded,
        Bound::Included(x) => Bound::Included(x.clone()),
        Bound::Excluded(x) => Bound::Excluded(x.clone()),
    }
}

impl Scene {
    pub fn hit<Bounds: RangeBounds<f64>>(
        &self,
        ray: Ray,
        range: Bounds,
    ) -> Option<Hit> {
        let mut res = None;
        let start: Bound<f64> = bound_cloned(range.start_bound());
        let mut end: Bound<f64> = bound_cloned(range.end_bound());

        for &sphere in &self.spheres {
            match sphere.hit(ray, (start, end)) {
                None => {}
                Some(hit) => {
                    end = Bound::Excluded(hit.t);
                    res = Some(hit);
                }
            }
        }

        res
    }
}

#[derive(Debug, Clone)]
pub struct Hit {
    pub point: Point,
    pub normal: Vector,
    pub t: f64,
    pub front_facing: bool,
    pub material: Material,
}

impl Hit {
    #[inline]
    fn new(ray: Ray, normal: Vector, t: f64, material: Material) -> Hit {
        let point = ray.origin + t * ray.dir;
        let front_facing = ray.dir.dot(&normal) < 0.0;
        Hit {
            point,
            normal: if front_facing { normal } else { -normal },
            t,
            front_facing,
            material,
        }
    }
}

pub trait Object {
    fn hit<Bounds: RangeBounds<f64>>(
        &self,
        ray: Ray,
        range: Bounds,
    ) -> Option<Hit>;
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub material: Material,
    pub center: Point,
    pub radius: f64,
}

impl Sphere {
    pub fn new(material: Material, center: Point, radius: f64) -> Sphere {
        Sphere {
            material,
            center,
            radius,
        }
    }
}

impl Object for Sphere {
    fn hit<Bounds: RangeBounds<f64>>(
        &self,
        ray: Ray,
        range: Bounds,
    ) -> Option<Hit> {
        let oc = ray.origin - self.center;
        // Solve the quadratic formula.
        let (a, b, c) = (
            ray.dir.dot(&ray.dir),
            2.0 * oc.dot(&ray.dir),
            oc.dot(&oc) - (self.radius.powi(2)),
        );
        let discriminant = b.powi(2) - (4.0 * a * c);
        if discriminant < 0.0 {
            None
        } else {
            // b^2 - 4ac > 0 ==> there is at least one root.
            // Subtract discriminant to find the smallest t such that
            // there's an intersection.
            let t: f64 = (-b - discriminant.sqrt()) / (2.0 * a);
            if range.contains(&t) {
                let point = ray.origin + t * ray.dir;
                let normal = (point - self.center).normalize();
                Some(Hit::new(ray, normal, t, self.material))
            } else {
                None
            }
        }
    }
}
