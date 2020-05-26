use crate::color::Color;
use crate::geometry::*;
use crate::pdf::PDF;
use crate::raytrace::Tracer;

const MAX_DEPTH: u32 = 25;

/// Unidirectional Path Tracer.
pub struct UDPT<'scene> {
    scene: &'scene Scene,
}

impl<'scene> UDPT<'scene> {
    pub fn new(scene: &'scene Scene) -> UDPT {
        UDPT { scene }
    }

    pub fn ray_color(&self, ray: Ray, depth: u32) -> Color {
        if depth > MAX_DEPTH {
            return Color::zeros();
        }
        match self.scene.hit(ray, 0.0001..f64::INFINITY) {
            Some(hit) => {
                let emit = hit.material.emitted(&hit);
                match hit.material.scatter(&ray, &hit) {
                    None => emit,
                    Some(scatter) => {
                        if let Some(specular) = scatter.specular {
                            return scatter.attenuation.component_mul(
                                &self.ray_color(specular, depth + 1),
                            );
                        }
                        if let Some(scatter_pdf) = scatter.pdf {
                            let light_pdf =
                                PDF::hittable(hit.point, &self.scene.lights);
                            let final_pdf = if self.scene.lights.is_empty() {
                                scatter_pdf
                            } else {
                                PDF::mix(0.75, &scatter_pdf, &light_pdf)
                            };
                            let scatter_ray = Ray {
                                origin: hit.point,
                                dir: final_pdf.gen(),
                            };
                            let color = self.ray_color(scatter_ray, depth + 1);
                            // The final value is the emission plus the MC
                            // estimate: attenuation
                            // * color(dir) * (s(dir) / p(dir))
                            // where `s` is the scattering PDF and `p` is the
                            // PDF we used to generate the direction.
                            // In Lambertian surfaces without MIS, for example,
                            // s(dir) and p(dir) are both cos(theta) where theta
                            // is the angle between the normal and dir, so those
                            // ordinarily cancel.
                            // Adding MIS mixes the final_pdf which we used to
                            // generate the scatter_ray, which throws off the
                            // calculation so we actually do need to do the
                            // division now.
                            return emit
                                + scatter.attenuation.component_mul(&color)
                                    * scatter_pdf.value(scatter_ray.dir)
                                    * final_pdf.value(scatter_ray.dir).recip();
                        }
                        Color::zeros()
                    }
                }
            }
            None => self.scene.background,
        }
    }
}

impl<'scene> Tracer for UDPT<'scene> {
    fn sample(&mut self, ray: Ray) -> Color {
        self.ray_color(ray, 0)
    }
}
