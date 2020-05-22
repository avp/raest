mod aabb;
mod bvh;
mod ray;
mod sphere;

pub use ray::Ray;

use crate::color::Color;
use crate::material::Material;
use crate::texture::Texture;
use crate::util::*;
use nalgebra::{Point3, Vector3};
use std::ops::Range;
use std::sync::Arc;

use aabb::AABB;
use bvh::BVHNode;
use sphere::Sphere;

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;

trait Hittable: Send + Sync {
    fn bounding_box(&self) -> AABB;
    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit>;
}

pub struct Scene {
    bvh: Arc<BVHNode>,
}

impl Scene {
    #[allow(dead_code)]
    fn from_objects(objects: Vec<Box<dyn Hittable>>) -> Scene {
        Scene {
            bvh: BVHNode::from_hittables(objects),
        }
    }

    #[inline]
    pub fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        self.bvh.hit(ray, range)
    }

    #[allow(dead_code)]
    pub fn random(n: u32) -> Scene {
        let mut objects: Vec<Box<dyn Hittable>> = vec![];
        let ground_texture = Texture::Checker(
            Box::new(Texture::Solid(Color::new(0.2, 0.3, 0.1))),
            Box::new(Texture::Solid(Color::new(0.9, 0.9, 0.9))),
        );
        let ground_material = Material::Lambertian(ground_texture);
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
                        material = Material::Lambertian(Texture::Solid(albedo));
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

        let material2 =
            Material::Lambertian(Texture::Solid(Color::new(0.4, 0.2, 0.1)));
        objects.push(Sphere::new(material2, Point::new(-4.0, 1.0, 0.0), 1.0));

        let material3 = Material::Metal(Color::new(0.7, 0.6, 0.5), 0.0);
        objects.push(Sphere::new(material3, Point::new(4.0, 1.0, 0.0), 1.0));

        Self::from_objects(objects)
    }

    #[allow(dead_code)]
    pub fn test() -> Scene {
        Scene::from_objects(vec![
            Sphere::new(
                Material::Lambertian(Texture::Solid(Color::new(0.7, 0.3, 0.3))),
                Point::new(0.0, 0.0, -1.0),
                0.5,
            ),
            Sphere::new(
                Material::Lambertian(Texture::Solid(Color::new(0.8, 0.8, 0.0))),
                Point::new(0.0, -100.5, -1.0),
                100.0,
            ),
            Sphere::new(
                Material::Metal(Color::new(0.8, 0.6, 0.2), 0.0),
                Point::new(1.0, 0.0, -1.0),
                0.5,
            ),
            Sphere::new(
                Material::Dielectric(1.5),
                Point::new(-1.0, 0.0, -1.0),
                0.5,
            ),
            Sphere::new(
                Material::Dielectric(1.5),
                Point::new(-1.0, 0.0, -1.0),
                -0.45,
            ),
        ])
    }
}

pub struct Hit<'obj> {
    pub point: Point,
    pub normal: Vector,
    pub t: f64,
    pub front_facing: bool,
    pub material: &'obj Material,
    pub uv: (f64, f64),
}

impl<'obj> Hit<'obj> {
    #[inline]
    fn new(
        ray: Ray,
        normal: Vector,
        t: f64,
        material: &'obj Material,
        uv: (f64, f64),
    ) -> Hit<'obj> {
        let point = ray.origin + t * ray.dir;
        let front_facing = ray.dir.dot(&normal) < 0.0;
        Hit {
            point,
            normal: if front_facing { normal } else { -normal },
            t,
            front_facing,
            material,
            uv,
        }
    }
}
