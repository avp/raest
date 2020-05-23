use crate::color::Color;
use crate::geometry::{Hit, Ray, ONB};
use crate::pdf::PDF;
use crate::texture::Texture;
use crate::util::*;
use nalgebra::Unit;
use std::f64::consts::PI;
use std::sync::Arc;

#[derive(Clone)]
pub enum Material {
    Lambertian(Arc<Texture>),
    Metal(Color, f64),
    Dielectric(f64),
    Emission(Arc<Texture>),
}

pub struct Scatter {
    pub ray: Ray,
    pub albedo: Color,
    pub pdf: f64,
}

impl Material {
    pub fn scatter(&self, inbound: &Ray, hit: &Hit) -> Option<Scatter> {
        match self {
            Material::Lambertian(albedo) => {
                let pdf = PDF::cosine(hit.normal);
                let scatter_dir = pdf.gen();
                Some(Scatter {
                    ray: Ray {
                        origin: hit.point,
                        dir: scatter_dir,
                    },
                    albedo: albedo.value(hit.uv, hit.point),
                    pdf: pdf.value(scatter_dir),
                })
            }
            &Material::Metal(albedo, roughness) => {
                let scatter_dir = reflect(inbound.dir.normalize(), hit.normal);
                Some(Scatter {
                    ray: Ray {
                        origin: hit.point,
                        dir: scatter_dir
                            + (roughness * random_in_unit_sphere()),
                    },
                    albedo,
                    pdf: 1.0,
                })
            }
            &Material::Dielectric(ior) => {
                let eta = if hit.front_facing { 1.0 / ior } else { ior };
                let dir = inbound.dir.normalize();

                let cos_theta = f64::min((-dir).dot(&hit.normal), 1.0);
                let sin_theta = (1.0 - (cos_theta * cos_theta)).sqrt();

                let scatter_dir = if (eta * sin_theta) > 1.0
                    || random_f64(0.0..1.0) < schlick(cos_theta, ior)
                {
                    // Must reflect.
                    reflect(dir, hit.normal)
                } else {
                    refract(dir, hit.normal, eta)
                };
                let out = Ray {
                    origin: hit.point,
                    dir: scatter_dir,
                };
                Some(Scatter {
                    ray: out,
                    albedo: Color::new(1.0, 1.0, 1.0),
                    pdf: 1.0,
                })
            }
            Material::Emission(..) => None,
        }
    }

    pub fn scatter_pdf(&self, _inbound: Ray, scattered: Ray, hit: &Hit) -> f64 {
        match self {
            Material::Lambertian(..) => {
                PDF::cosine(hit.normal).value(scattered.dir)
            }
            Material::Metal(..) => 1.0,
            Material::Dielectric(..) => 1.0,
            Material::Emission(..) => 0.0,
        }
    }

    pub fn emitted(&self, hit: &Hit) -> Color {
        match self {
            Material::Lambertian(..) => Color::zeros(),
            Material::Metal(..) => Color::zeros(),
            Material::Dielectric(..) => Color::zeros(),
            Material::Emission(tex) => tex.value(hit.uv, hit.point),
        }
    }
}
