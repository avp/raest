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
    #[allow(dead_code)]
    pub fn new(scene: &'scene Scene) -> UDPT {
        UDPT { scene }
    }

    pub fn ray_color(&self, ray: Ray, depth: u32) -> Color {
        if depth == 0 {
            return Color::zeros();
        }
        let mut ray = ray;
        let mut result = Color::new(1.0, 1.0, 1.0);
        for _ in 0..depth {
            match self.scene.hit(ray, 0.0001..f64::INFINITY) {
                Some(hit) => {
                    let emit = hit.material.emitted(&hit);
                    match hit.material.scatter(&ray, &hit) {
                        None => {
                            result = result.component_mul(&emit);
                            break;
                        }
                        Some(scatter) => {
                            if let Some(specular) = scatter.specular {
                                result =
                                    result.component_mul(&scatter.attenuation);
                                ray = specular;
                            }
                            if let Some(scatter_pdf) = scatter.pdf {
                                let light_pdf = PDF::hittable(
                                    hit.point,
                                    &self.scene.lights,
                                );
                                let final_pdf = if self.scene.lights.is_empty()
                                {
                                    scatter_pdf
                                } else {
                                    PDF::mix(0.75, &scatter_pdf, &light_pdf)
                                };
                                let scatter_ray = Ray {
                                    origin: hit.point,
                                    dir: final_pdf.gen(),
                                };
                                // The final value is the emission plus the MC
                                // estimate: attenuation
                                // * color(dir) * (s(dir) / p(dir))
                                // where `s` is the scattering PDF and `p` is
                                // the
                                // PDF we used to generate the direction.
                                // In Lambertian surfaces without MIS, for
                                // example,
                                // s(dir) and p(dir) are both cos(theta) where
                                // theta
                                // is the angle between the normal and dir, so
                                // those
                                // ordinarily cancel.
                                // Adding MIS mixes the final_pdf which we used
                                // to
                                // generate the scatter_ray, which throws off
                                // the
                                // calculation so we actually do need to do the
                                // division now.
                                result = result
                                    .component_mul(&scatter.attenuation)
                                    * scatter_pdf.value(scatter_ray.dir)
                                    * final_pdf.value(scatter_ray.dir).recip();
                                ray = scatter_ray;
                            }
                            Color::zeros()
                        }
                    }
                }
                None => {
                    result = result.component_mul(&self.scene.background);
                    break;
                }
            };
        }
        result
    }
}

impl<'scene> Tracer for UDPT<'scene> {
    fn sample(&mut self, ray: Ray, _debug: bool) -> Color {
        self.ray_color(ray, MAX_DEPTH)
    }
}
