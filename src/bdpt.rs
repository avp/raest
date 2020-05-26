use crate::color::Color;
use crate::geometry::*;
use crate::material::Scatter;
use crate::pdf::PDF;
use crate::raytrace::Tracer;
use nalgebra::Unit;

/// Bidirectional Path Tracer.
pub struct BDPT<'s> {
    scene: &'s Scene,
}

const MAX_CAMERA_DEPTH: u32 = 10;
const MAX_LIGHT_DEPTH: u32 = 10;

impl<'s> BDPT<'s> {
    #[allow(dead_code)]
    pub fn new(scene: &'s Scene) -> BDPT {
        BDPT { scene }
    }

    fn ray_color(&self, ray: Ray, _debug: bool) -> Color {
        let camera_path = self.random_walk(
            WalkKind::Camera,
            ray,
            Color::new(1.0, 1.0, 1.0),
            MAX_CAMERA_DEPTH,
        );
        let light_path = self.gen_light_path();
        let mut result = Color::zeros();
        if camera_path.len() == 1 {
            return self.scene.background;
        }
        for t in 1..=camera_path.len() - 1 {
            for s in 0..=light_path.len() - 1 {
                let w = (s + t + 1) as f64;
                result +=
                    self.join_path(&camera_path, &light_path, t, s) * w.recip();
            }
        }
        result
    }

    /// Use vertices 0..=t of the camera path and 0..=s vertices of the light
    /// path.
    fn join_path(
        &self,
        camera_path: &[Vertex],
        light_path: &[Vertex],
        t: usize,
        s: usize,
    ) -> Color {
        if t > 0 {
            if !self.test_visibility(&camera_path[t], &light_path[s]) {
                return Color::zeros();
            }
        }
        // let mut result = Color::new(1.0, 1.0, 1.0);
        // for i in 1..=s {
        //     match &camera_path[i] {
        //         Vertex::Light(..) => {
        //             result *= 15.0;
        //             break;
        //         }
        //         Vertex::Specular { scatter, .. } => {
        //             result = result.component_mul(&scatter.attenuation);
        //         }
        //         Vertex::Scatter {
        //             scatter,
        //             pdf_fwd,
        //             scatter_ray,
        //             ..
        //         } => {
        //             result = result.component_mul(&scatter.attenuation);
        //             result *= scatter.pdf.unwrap().value(scatter_ray.dir);
        //             result *= pdf_fwd.recip();
        //         }
        //         Vertex::Camera(..) => unreachable!(),
        //     }
        // }
        let mut result = camera_path[t].beta();
        // if (result.x - camera_path[s].beta().x).abs() > 1.0 {
        //     dbg!(s, camera_path[s].beta() - result);
        //     let _: Vec<Color> =
        //         dbg!(camera_path.iter().map(|v| v.beta()).collect());
        //     unreachable!();
        // }
        for i in (0..=s).rev() {
            match &light_path[i] {
                Vertex::Light(_, color) => {
                    result = result.component_mul(&color)
                }
                Vertex::Specular { scatter, .. } => {
                    result = result.component_mul(&scatter.attenuation);
                }
                Vertex::Scatter {
                    scatter,
                    pdf_fwd,
                    scatter_ray,
                    ..
                } => {
                    result = result.component_mul(&scatter.attenuation);
                    result *= scatter.pdf.unwrap().value(scatter_ray.dir);
                    result *= pdf_fwd.recip();
                }
                Vertex::Camera(..) => unreachable!(),
            }
        }
        result
    }

    fn gen_light_path(&self) -> Vec<Vertex> {
        let (ray, color) = self.scene.lights.emit();
        self.random_walk(WalkKind::Light, ray, color, MAX_LIGHT_DEPTH)
    }

    fn random_walk(
        &self,
        kind: WalkKind,
        ray: Ray,
        mut beta: Color,
        mut depth: u32,
    ) -> Vec<Vertex> {
        if depth == 0 {
            return vec![];
        }

        let mut ray = ray;
        let mut result: Vec<Vertex> = vec![match kind {
            WalkKind::Camera => Vertex::camera(ray),
            WalkKind::Light => Vertex::light(ray, beta),
        }];
        depth -= 1;

        while result.len() < depth as usize {
            let prev = result.last_mut().unwrap();
            match self.scene.hit(ray, 0.0001..f64::INFINITY) {
                None => break,
                Some(hit) => {
                    let emit = hit.material.emitted(&hit);
                    if emit.norm_squared() > 0.0 {
                        beta = beta.component_mul(&emit);
                        result.push(Vertex::Light(
                            Ray {
                                origin: hit.point,
                                dir: *hit.normal,
                            },
                            beta,
                        ));
                        break;
                    }
                    if let Some(scatter) = hit.material.scatter(&ray, &hit) {
                        if let Some(specular) = scatter.specular {
                            ray = specular;
                            result.push(Vertex::specular(hit, scatter));
                            continue;
                        }
                        if let Some(scatter_pdf) = scatter.pdf {
                            let light_pdf =
                                PDF::hittable(hit.point, &self.scene.lights);
                            let final_pdf = scatter_pdf;
                            let final_pdf = match kind {
                                WalkKind::Camera => {
                                    if self.scene.lights.is_empty() {
                                        scatter_pdf
                                    } else {
                                        PDF::mix(0.75, &scatter_pdf, &light_pdf)
                                    }
                                }
                                WalkKind::Light => scatter_pdf,
                            };
                            let scatter_ray = Ray {
                                origin: hit.point,
                                dir: final_pdf.gen(),
                            };
                            ray = scatter_ray;
                            let _g = self.g(prev, &hit);
                            let pdf_fwd = final_pdf.value(scatter_ray.dir);
                            beta = beta.component_mul(&scatter.attenuation);
                            beta *= scatter_pdf.value(scatter_ray.dir);
                            beta *= pdf_fwd.recip();
                            result.push(Vertex::scatter(
                                hit,
                                scatter,
                                pdf_fwd,
                                beta,
                                scatter_ray,
                            ));
                            continue;
                        }
                    }
                }
            }
        }
        result
    }

    fn test_visibility(&self, v1: &Vertex, v2: &Vertex) -> bool {
        const EPS: f64 = 0.001;
        let (p1, p2) = (
            v1.point() + EPS * *v1.normal(),
            v2.point() + EPS * *v2.normal(),
        );
        let ray = Ray {
            origin: p1,
            dir: p2 - p1,
        };
        let tmin = EPS;
        let tmax = ray.dir.norm() - EPS;
        self.scene.hit(ray, tmin..tmax).is_none()
    }

    /// g_i(y) term in the light transport equation.
    /// g_i(y) = cos(theta_i) * cos(theta_{i+1}) / || y_i - y_{i-1} ||^2
    /// where thetas are the angle between surface normals and the segment
    /// connecting the vertices.
    fn g(&self, v1: &Vertex, v2: &Hit) -> f64 {
        let (p1, p2) = (v1.point(), v2.point);
        let d = p2 - p1;
        let d_normalized = d.normalize();
        let cos_theta1 = v1.normal().dot(&d_normalized);
        let cos_theta2 = v2.normal.dot(&-d_normalized);
        cos_theta1 * cos_theta2 / d.norm_squared()
    }
}

impl<'scene> Tracer for BDPT<'scene> {
    fn sample(&mut self, ray: Ray, debug: bool) -> Color {
        self.ray_color(ray, debug)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum WalkKind {
    Camera,
    Light,
}

enum Vertex<'s> {
    Camera(Ray),
    Light(Ray, Color),
    Specular {
        hit: Hit<'s>,
        scatter: Scatter<'s>,
    },
    Scatter {
        hit: Hit<'s>,
        scatter: Scatter<'s>,
        pdf_fwd: f64,
        pdf_rev: f64,
        beta: Color,
        scatter_ray: Ray,
    },
}

impl<'s> Vertex<'s> {
    pub fn camera(ray: Ray) -> Vertex<'s> {
        Vertex::Camera(ray)
    }
    pub fn light(ray: Ray, color: Color) -> Vertex<'s> {
        Vertex::Light(ray, color)
    }
    pub fn specular(hit: Hit<'s>, scatter: Scatter<'s>) -> Vertex<'s> {
        Vertex::Specular { hit, scatter }
    }
    pub fn scatter(
        hit: Hit<'s>,
        scatter: Scatter<'s>,
        pdf_fwd: f64,
        beta: Color,
        scatter_ray: Ray,
    ) -> Vertex<'s> {
        Vertex::Scatter {
            hit,
            scatter,
            pdf_fwd,
            beta,
            pdf_rev: 0.0,
            scatter_ray,
        }
    }
    pub fn point(&self) -> Point {
        match self {
            Vertex::Camera(ray) => ray.origin,
            Vertex::Light(ray, ..) => ray.origin,
            Vertex::Specular { hit, .. } => hit.point,
            Vertex::Scatter { hit, .. } => hit.point,
        }
    }
    pub fn normal(&self) -> Unit<Vector> {
        match self {
            Vertex::Camera(ray) => Unit::new_normalize(ray.dir),
            Vertex::Light(ray, ..) => Unit::new_normalize(ray.dir),
            Vertex::Specular { hit, .. } => hit.normal,
            Vertex::Scatter { hit, .. } => hit.normal,
        }
    }
    pub fn beta(&self) -> Color {
        match self {
            Vertex::Camera(_) => Color::zeros(),
            Vertex::Light(_, color) => *color,
            Vertex::Specular { .. } => Color::zeros(),
            Vertex::Scatter { beta, .. } => *beta,
        }
    }
}
