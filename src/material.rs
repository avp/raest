use crate::color::Color;
use crate::geometry::{Hit, Ray};
use crate::texture::Texture;
use crate::util::*;

#[derive(Clone)]
pub enum Material {
    Lambertian(Texture),
    Metal(Color, f64),
    Dielectric(f64),
    Emission(Texture),
}

impl Material {
    pub fn scatter(&self, inbound: &Ray, hit: &Hit) -> Option<(Ray, Color)> {
        match self {
            Material::Lambertian(albedo) => {
                let scatter_dir =
                    hit.normal + random_in_unit_sphere().normalize();
                Some((
                    Ray {
                        origin: hit.point,
                        dir: scatter_dir,
                    },
                    albedo.value(hit.uv, hit.point),
                ))
            }
            &Material::Metal(albedo, roughness) => {
                let scatter_dir = reflect(inbound.dir.normalize(), hit.normal);
                Some((
                    Ray {
                        origin: hit.point,
                        dir: scatter_dir
                            + (roughness * random_in_unit_sphere()),
                    },
                    albedo,
                ))
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
                Some((out, Color::new(1.0, 1.0, 1.0)))
            }
            Material::Emission(..) => None,
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
