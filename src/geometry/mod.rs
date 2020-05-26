mod aabb;
mod block;
mod bvh;
mod onb;
mod parser;
mod ray;
mod rect;
mod sphere;
mod transform;

pub use onb::ONB;
pub use ray::Ray;

use crate::camera::Camera;
use crate::color::Color;
use crate::config::Config;
use crate::material::Material;
use crate::texture::Texture;
use crate::util::*;
use nalgebra::{Point3, Unit, Vector3};
use rand::seq::SliceRandom;
use std::ops::Range;
use std::sync::Arc;

use aabb::AABB;
use block::Block;
use bvh::BVHNode;
pub use rect::{Rect, RectAxis};
use sphere::Sphere;
use transform::{Rotate, Translate};

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;

pub trait Hittable: Send + Sync {
    fn is_light(&self) -> bool;
    fn bounding_box(&self) -> AABB;
    fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit>;
    fn pdf(&self, _ray: Ray) -> f64 {
        eprintln!("Warning: Attempting to sample PDF for unimplemented object");
        0.0
    }
    fn random(&self, _origin: Point) -> Vector {
        eprintln!("Warning: Attempting to sample PDF for unimplemented object");
        Vector::x()
    }
    fn emit(&self) -> (Ray, Color) {
        unimplemented!("Not all objects implement emit() yet");
    }
}

pub struct Scene {
    pub background: Color,
    bvh: Arc<BVHNode>,
    pub lights: HittableList,
}

impl Scene {
    #[allow(dead_code)]
    fn from_objects(
        background: Color,
        objects: Vec<Arc<dyn Hittable>>,
    ) -> Scene {
        let mut lights = vec![];
        for obj in &objects {
            if obj.is_light() {
                lights.push(obj.clone());
            }
        }
        Scene {
            background,
            bvh: BVHNode::from_hittables(objects),
            lights: HittableList::new(lights),
        }
    }

    #[inline]
    pub fn hit(&self, ray: Ray, range: Range<f64>) -> Option<Hit> {
        self.bvh.hit(ray, range)
    }

    #[inline(never)]
    pub fn from_config(config: &Config) -> (Scene, Camera) {
        parser::parse(&config)
    }

    #[allow(dead_code)]
    pub fn random(config: &Config, n: u32) -> (Scene, Camera) {
        let mut objects: Vec<Arc<dyn Hittable>> = vec![];
        let ground_texture = Arc::new(Texture::Checker(
            Arc::new(Texture::Solid(Color::new(0.2, 0.3, 0.1))),
            Arc::new(Texture::Solid(Color::new(0.9, 0.9, 0.9))),
        ));
        let ground_material = Arc::new(Material::Lambertian(ground_texture));
        objects.push(Sphere::new(
            ground_material.clone(),
            Point::new(0.0, -1000.0, 0.0),
            1000.0,
        ));

        let count = n as i64;

        for a in -count..count {
            for b in -count..count {
                let mat_rand = random();
                let center = Point::new(
                    a as f64 + 0.9 * random(),
                    0.2,
                    b as f64 + 0.9 * random(),
                );

                if (center - Vector::new(4.0, 0.2, 0.0)).coords.norm() > 0.9 {
                    let material;

                    if mat_rand < 0.8 {
                        let albedo = Color::new(random(), random(), random());
                        material = Arc::new(Material::Lambertian(Arc::new(
                            Texture::Solid(albedo),
                        )));
                    } else if mat_rand < 0.95 {
                        let albedo = Color::new(
                            random_range(0.5..1.0),
                            random_range(0.5..1.0),
                            random_range(0.5..1.0),
                        );
                        let fuzz = random_range(0.0..0.5);
                        material = Arc::new(Material::Metal(albedo, fuzz));
                    } else {
                        material = Arc::new(Material::Dielectric(1.5));
                    }

                    objects.push(Sphere::new(material, center, 0.2));
                }
            }
        }

        let material1 = Arc::new(Material::Dielectric(1.5));
        objects.push(Sphere::new(material1, Point::new(0.0, 1.0, 0.0), 1.0));

        let material2 = Arc::new(Material::Lambertian(Arc::new(
            Texture::Solid(Color::new(0.4, 0.2, 0.1)),
        )));
        objects.push(Sphere::new(material2, Point::new(-4.0, 1.0, 0.0), 1.0));

        let material3 =
            Arc::new(Material::Metal(Color::new(0.7, 0.6, 0.5), 0.0));
        objects.push(Sphere::new(material3, Point::new(4.0, 1.0, 0.0), 1.0));

        let aspect_ratio: f64 = config.width as f64 / config.height as f64;
        let from = Point::new(13.0, 2.0, 8.0);
        let at = Point::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        let dist = 10.0;
        let camera = Camera::new(from, at, up, 20.0, aspect_ratio, 0.1, dist);

        (
            Self::from_objects(Color::new(0.5, 0.7, 1.0), objects),
            camera,
        )
    }

    #[allow(dead_code)]
    pub fn earth(config: &Config) -> (Scene, Camera) {
        let aspect_ratio: f64 = config.width as f64 / config.height as f64;
        let from = Point::new(13.0, 2.0, 8.0);
        let at = Point::new(0.0, 0.0, 0.0);
        let up = Vector::new(0.0, 1.0, 0.0);
        let dist = 10.0;
        let camera = Camera::new(from, at, up, 20.0, aspect_ratio, 0.1, dist);

        let earth_tex = Arc::new(Texture::Image(
            image::open("images/earthmap.jpg").unwrap(),
        ));
        let global = Sphere::new(
            Arc::new(Material::Lambertian(earth_tex)),
            Point::origin(),
            2.0,
        );
        (
            Scene::from_objects(Color::new(0.5, 0.7, 1.0), vec![global]),
            camera,
        )
    }
}

pub struct Hit<'obj> {
    pub point: Point,
    pub normal: Unit<Vector>,
    pub t: f64,
    pub front_facing: bool,
    pub material: &'obj Material,
    pub uv: (f64, f64),
}

impl<'obj> Hit<'obj> {
    #[inline]
    fn new(
        ray: Ray,
        normal: Unit<Vector>,
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

pub struct HittableList {
    hittables: Vec<Arc<dyn Hittable>>,
}

impl HittableList {
    pub fn new(hittables: Vec<Arc<dyn Hittable>>) -> HittableList {
        HittableList { hittables }
    }

    pub fn is_empty(&self) -> bool {
        self.hittables.is_empty()
    }
}

impl Hittable for HittableList {
    fn is_light(&self) -> bool {
        false
    }

    fn bounding_box(&self) -> AABB {
        if self.is_empty() {
            return AABB::new(Point::origin(), Point::origin());
        }
        let mut result = self.hittables[0].bounding_box();
        for h in &self.hittables {
            result = AABB::containing(result, h.bounding_box());
        }
        result
    }

    fn hit(&self, ray: Ray, mut range: Range<f64>) -> Option<Hit> {
        let mut result: Option<Hit> = None;
        for h in &self.hittables {
            if let Some(hit) = h.hit(ray, range.clone()) {
                range.end = hit.t;
                result = Some(hit);
            }
        }
        result
    }

    fn pdf(&self, ray: Ray) -> f64 {
        let weight = (self.hittables.len() as f64).recip();
        let mut sum = 0.0;
        for h in &self.hittables {
            sum += weight * h.pdf(ray);
        }
        sum
    }

    fn random(&self, origin: Point) -> Vector {
        let h = &self
            .hittables
            .as_slice()
            .choose(&mut rand::thread_rng())
            .unwrap();
        h.random(origin)
    }

    fn emit(&self) -> (Ray, Color) {
        let h = &self
            .hittables
            .as_slice()
            .choose(&mut rand::thread_rng())
            .unwrap();
        h.emit()
    }
}
