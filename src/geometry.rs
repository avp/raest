use crate::bvh::BVHNode;
use crate::color::Color;
use crate::util::*;
use nalgebra::{Point3, Vector3};
use std::ops::Range;
use std::sync::Arc;

pub use crate::aabb::AABB;
pub use crate::material::Material;
pub use crate::ray::Ray;

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;

pub trait Hittable: Send + Sync {
    fn bounding_box(&self) -> AABB;
    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit>;
}

pub struct Scene {
    bvh: Arc<BVHNode>,
}

impl Scene {
    #[allow(dead_code)]
    pub fn from_objects(objects: Vec<Box<dyn Hittable>>) -> Scene {
        Scene {
            bvh: BVHNode::from_hittables(objects),
        }
    }

    #[allow(dead_code)]
    pub fn random(n: u32) -> Scene {
        let mut objects: Vec<Box<dyn Hittable>> = vec![];
        let ground_material = Material::Lambertian(Color::new(0.5, 0.5, 0.5));
        objects.push(Sphere::new(
            ground_material,
            Point::new(0.0, -1000.0, 0.0),
            1000.0,
        ));

        let count = n as i64;

        for a in -count..count {
            for b in -count..count {
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

                    objects.push(Sphere::new(material, center, 0.2));
                }
            }
        }

        let material1 = Material::Dielectric(1.5);
        objects.push(Sphere::new(material1, Point::new(0.0, 1.0, 0.0), 1.0));

        let material2 = Material::Lambertian(Color::new(0.4, 0.2, 0.1));
        objects.push(Sphere::new(material2, Point::new(-4.0, 1.0, 0.0), 1.0));

        let material3 = Material::Metal(Color::new(0.7, 0.6, 0.5), 0.0);
        objects.push(Sphere::new(material3, Point::new(4.0, 1.0, 0.0), 1.0));

        Scene {
            bvh: BVHNode::from_hittables(objects),
        }
    }

    #[inline]
    pub fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        self.bvh.hit(ray, range)
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

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub material: Material,
    pub center: Point,
    pub radius: f64,
}

impl Sphere {
    pub fn new(material: Material, center: Point, radius: f64) -> Box<Sphere> {
        Box::new(Sphere {
            material,
            center,
            radius,
        })
    }
}

impl Hittable for Sphere {
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
            let normal = (point - self.center) * (1.0 / self.radius);
            Some(Hit::new(ray, normal, t, self.material))
        }
    }
}
