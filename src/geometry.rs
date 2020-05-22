use crate::color::Color;
use crate::util::*;
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
    pub fn random() -> Scene {
        let mut spheres = vec![];
        let ground_material = Material::Lambertian(Color::new(0.5, 0.5, 0.5));
        spheres.push(Sphere::new(
            ground_material,
            Point::new(0.0, -1000.0, 0.0),
            1000.0,
        ));

        for a in -11..11 {
            for b in -11..11 {
                let mat_rand = random_f64(0.0..1.0);
                let center = Point::new(
                    a as f64 + 0.9 * random_f64(0.0..1.0),
                    0.2,
                    b as f64 + 0.9 * random_f64(0.0..1.0),
                );

                if (center - Vector::new(4.0, 0.2, 0.0)).coords.norm() > 0.9 {
                    let material;

                    if mat_rand < 0.8 {
                        let albedo = Color::new(
                            random_f64(0.0..1.0),
                            random_f64(0.0..1.0),
                            random_f64(0.0..1.0),
                        );
                        material = Material::Lambertian(albedo);
                    } else if mat_rand < 0.95 {
                        let albedo = Color::new(
                            random_f64(0.5..1.0),
                            random_f64(0.5..1.0),
                            random_f64(0.5..1.0),
                        );
                        let fuzz = random_f64(0.0..0.5);
                        material = Material::Metal(albedo, fuzz);
                    } else {
                        material = Material::Dielectric(1.5);
                    }

                    spheres.push(Sphere::new(material, center, 0.2));
                }
            }
        }

        let material1 = Material::Dielectric(1.5);
        spheres.push(Sphere::new(material1, Point::new(0.0, 1.0, 0.0), 1.0));

        let material2 = Material::Lambertian(Color::new(0.4, 0.2, 0.1));
        spheres.push(Sphere::new(material2, Point::new(-4.0, 1.0, 0.0), 1.0));

        let material3 = Material::Metal(Color::new(0.7, 0.6, 0.5), 0.0);
        spheres.push(Sphere::new(material3, Point::new(4.0, 1.0, 0.0), 1.0));

        Scene { spheres }
    }
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
            let t1: f64 = (-half_b - discriminant.sqrt()) / a;
            let t2: f64 = (-half_b + discriminant.sqrt()) / a;
            let t = if range.contains(&t1) {
                t1
            } else if range.contains(&t2) {
                t2
            } else {
                return None;
            };
            let point = ray.at(t);
            let normal = (point - self.center) * (1.0 / self.radius);
            Some(Hit::new(ray, normal, t, self.material))
        }
    }
}
