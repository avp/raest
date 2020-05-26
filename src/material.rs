use crate::color::Color;
use crate::geometry::{Hit, Ray};
use crate::pdf::PDF;
use crate::texture::Texture;
use crate::util::*;
use nalgebra::Unit;
use std::sync::Arc;

#[derive(Clone)]
pub enum Material {
    Lambertian(Arc<Texture>),
    Phong(f64, Arc<Texture>, Arc<Texture>, u32),
    Metal(Color, f64),
    Dielectric(f64),
    Emission(Arc<Texture>),
}

pub struct Scatter<'scene> {
    pub specular: Option<Ray>,
    pub pdf: Option<PDF<'scene>>,
    pub attenuation: Color,
}

impl Material {
    pub fn scatter(&self, inbound: &Ray, hit: &Hit) -> Option<Scatter> {
        match self {
            Material::Lambertian(albedo) => {
                let pdf = PDF::cosine(hit.normal);
                Some(Scatter {
                    attenuation: albedo.value(hit.uv, hit.point),
                    pdf: Some(pdf),
                    specular: None,
                })
            }
            Material::Phong(kd, diffuse, specular, shininess) => {
                if random() < *kd {
                    // Diffuse, use standard cosine PDF.
                    let pdf = PDF::cosine(hit.normal);
                    Some(Scatter {
                        attenuation: diffuse.value(hit.uv, hit.point),
                        pdf: Some(pdf),
                        specular: None,
                    })
                } else {
                    // Specular, use reflected ray as the normal.
                    let reflect_dir = Unit::new_unchecked(reflect(
                        inbound.dir.normalize(),
                        hit.normal,
                    ));
                    let pdf = PDF::phong(hit.normal, reflect_dir, *shininess);
                    Some(Scatter {
                        attenuation: specular.value(hit.uv, hit.point),
                        pdf: Some(pdf),
                        specular: None,
                    })
                }
            }
            &Material::Metal(albedo, roughness) => {
                let scatter_dir = reflect(inbound.dir.normalize(), hit.normal);
                Some(Scatter {
                    specular: Some(Ray {
                        origin: hit.point,
                        dir: scatter_dir
                            + (roughness * random_in_unit_sphere()),
                    }),
                    attenuation: albedo,
                    pdf: None,
                })
            }
            &Material::Dielectric(ior) => {
                let eta = if hit.front_facing { 1.0 / ior } else { ior };
                let dir = inbound.dir.normalize();

                let cos_theta = f64::min((-dir).dot(&hit.normal), 1.0);
                let sin_theta = (1.0 - (cos_theta * cos_theta)).sqrt();

                let scatter_dir = if (eta * sin_theta) > 1.0
                    || random() < schlick(cos_theta, ior)
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
                    specular: Some(out),
                    attenuation: Color::new(1.0, 1.0, 1.0),
                    pdf: None,
                })
            }
            Material::Emission(..) => None,
        }
    }

    pub fn emitted(&self, hit: &Hit) -> Color {
        match self {
            Material::Lambertian(..) => Color::zeros(),
            Material::Phong(..) => Color::zeros(),
            Material::Metal(..) => Color::zeros(),
            Material::Dielectric(..) => Color::zeros(),
            Material::Emission(tex) => tex.value(hit.uv, hit.point),
        }
    }
}
