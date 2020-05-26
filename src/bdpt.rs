use crate::color::Color;
use crate::geometry::*;
use crate::pdf::PDF;
use crate::raytrace::Tracer;

/// Bidirectional Path Tracer.
pub struct BDPT<'s> {
    scene: &'s Scene,
}

impl<'s> BDPT<'s> {
    #[allow(dead_code)]
    pub fn new(scene: &'s Scene) -> BDPT {
        BDPT { scene }
    }

    fn ray_color(&self, ray: Ray) -> Color {
        let camera_path = self.random_walk(WalkKind::Camera, ray, 8);
        let light_path = self.gen_light_path();
        unimplemented!()
    }

    fn gen_light_path(&self) -> Vec<Vertex<'s>> {
        let ray = self.scene.lights.emit();
        self.random_walk(WalkKind::Light, ray, 8)
    }

    fn random_walk(
        &self,
        kind: WalkKind,
        ray: Ray,
        depth: u32,
    ) -> Vec<Vertex<'s>> {
        if depth == 0 {
            return vec![];
        }

        let mut ray = ray;
        let mut result: Vec<Vertex> = vec![match kind {
            WalkKind::Camera => Vertex::camera(ray.origin),
            WalkKind::Light => Vertex::light(ray.origin),
        }];

        while result.len() < depth as usize {
            match self.scene.hit(ray, 0.0001..f64::INFINITY) {
                None => break,
                Some(hit) => {
                    let emit = hit.material.emitted(&hit);
                    if emit.norm_squared() > 0.0 {
                        result.push(Vertex::Light(hit.point));
                        break;
                    }
                    if let Some(scatter) = hit.material.scatter(&ray, &hit) {
                        if let Some(specular) = scatter.specular {
                            ray = specular;
                            result.push(Vertex::specular(hit));
                            continue;
                        }
                        if let Some(scatter_pdf) = scatter.pdf {
                            let final_pdf = scatter_pdf;
                            let scatter_ray = Ray {
                                origin: hit.point,
                                dir: final_pdf.gen(),
                            };
                            ray = scatter_ray;
                            result.push(Vertex::scatter(hit, final_pdf));
                            continue;
                        }
                    }
                }
            }
        }
        result
    }
}

impl<'scene> Tracer for BDPT<'scene> {
    fn sample(&mut self, ray: Ray) -> Color {
        self.ray_color(ray)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum WalkKind {
    Camera,
    Light,
}

enum Vertex<'s> {
    Camera(Point),
    Light(Point),
    Specular { hit: Hit<'s> },
    Scatter { hit: Hit<'s>, pdf: PDF<'s> },
}

impl<'s> Vertex<'s> {
    pub fn camera(point: Point) -> Vertex<'s> {
        Vertex::Camera(point)
    }
    pub fn light(point: Point) -> Vertex<'s> {
        Vertex::Light(point)
    }
    pub fn specular(hit: Hit<'s>) -> Vertex<'s> {
        Vertex::Specular { hit }
    }
    pub fn scatter(hit: Hit<'s>, pdf: PDF<'s>) -> Vertex<'s> {
        Vertex::Scatter { hit, pdf }
    }
}
