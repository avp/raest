use crate::color::Color;
use crate::geometry::{Hit, Ray};
use crate::util::*;

#[derive(Debug, Copy, Clone)]
pub enum Material {
    Lambertian(Color),
    Metal(Color, f64),
}

impl Material {
    pub fn scatter(&self, inbound: &Ray, hit: &Hit) -> (Ray, Color) {
        match *self {
            Material::Lambertian(albedo) => {
                let scatter_dir = hit.normal + random_in_unit_sphere();
                (
                    Ray {
                        origin: hit.point,
                        dir: scatter_dir,
                    },
                    albedo,
                )
            }
            Material::Metal(albedo, roughness) => {
                let scatter_dir = reflect(inbound.dir.normalize(), hit.normal);
                (
                    Ray {
                        origin: hit.point,
                        dir: scatter_dir
                            + (roughness * random_in_unit_sphere()),
                    },
                    albedo,
                )
            }
        }
    }
}
