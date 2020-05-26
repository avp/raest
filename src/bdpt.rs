use crate::color::Color;
use crate::geometry::*;
use crate::raytrace::Tracer;
use crate::util::*;
use nalgebra::Unit;
use std::f64::consts::PI;

/// Bidirectional Path Tracer.
pub struct BDPT<'s> {
    scene: &'s Scene,
}

const MIN_DEPTH: u32 = 3;
const MAX_CAMERA_DEPTH: u32 = 10;
const MAX_LIGHT_DEPTH: u32 = 10;
const EPS: f64 = 0.00001;

impl<'s> BDPT<'s> {
    #[allow(dead_code)]
    pub fn new(scene: &'s Scene) -> BDPT {
        BDPT { scene }
    }

    fn ray_color(&self, ray: Ray, _debug: bool) -> Color {
        let camera_path = self.random_walk(
            WalkKind::Camera,
            ray,
            Unit::new_normalize(ray.dir),
            Color::new(1.0, 1.0, 1.0),
            MAX_CAMERA_DEPTH,
        );
        let mut result = Color::zeros();
        if camera_path.len() == 1 {
            return self.scene.background;
        }
        if camera_path.len() == 2 && camera_path[1].kind == VertexKind::Light {
            return camera_path[1].beta;
        }
        let light_path = self.gen_light_path();
        for t in 2..=camera_path.len() - 1 {
            for s in 0..=light_path.len() - 1 {
                result += self.join_path(&camera_path, &light_path, t, s);
            }
        }
        result
    }

    /// Use t vertices of the camera path and s vertices of the light path.
    fn join_path(
        &self,
        camera_path: &[Vertex],
        light_path: &[Vertex],
        t: usize,
        s: usize,
    ) -> Color {
        if s > 0 {
            if !self.test_visibility(&camera_path[t - 1], &light_path[s - 1]) {
                return Color::zeros();
            }
        }
        let (w_camera, w_light) = if t > 1 && s > 1 {
            (
                (camera_path[t - 1].pdf_rev * camera_path[t - 1].g_rev)
                    * (camera_path[t - 1].vcm
                        * camera_path[t - 2].pdf_rev
                        * camera_path[t - 1].vc),
                (light_path[s - 1].pdf_rev * light_path[s - 1].g_rev)
                    * (light_path[s - 1].vcm
                        * light_path[s - 2].pdf_rev
                        * light_path[s - 1].vc),
            )
        } else if s == 1 {
            (
                (camera_path[t - 1].pdf_rev * camera_path[t - 1].g_rev)
                    * (camera_path[t - 1].vcm
                        * camera_path[t - 2].pdf_rev
                        * camera_path[t - 1].vc),
                (light_path[0].pdf_rev * light_path[0].g_rev)
                    / (light_path[0].pdf_fwd * light_path[0].g_fwd),
            )
        } else if s == 0 {
            assert!(t > 1);
            (
                (camera_path[t - 1].pdf_rev * camera_path[t - 1].g_rev)
                    * (camera_path[t - 1].vcm
                        * camera_path[t - 2].pdf_rev
                        * camera_path[t - 1].vc),
                0.0,
            )
        } else {
            unreachable!();
        };
        let (vt, vs) = (
            camera_path[t - 1].beta,
            if s > 0 {
                light_path[s - 1].beta
            } else {
                Color::new(1.0, 1.0, 1.0)
            },
        );
        let w_st = (w_camera + 1.0 + w_light).recip();
        (vt).component_mul(&(vs)) * w_st
    }

    fn gen_light_path(&self) -> Vec<Vertex> {
        let (ray, normal, color) = self.scene.lights.emit();
        self.random_walk(WalkKind::Light, ray, normal, color, MAX_LIGHT_DEPTH)
    }

    fn random_walk(
        &self,
        kind: WalkKind,
        ray: Ray,
        normal: Unit<Vector>,
        mut beta: Color,
        mut depth: u32,
    ) -> Vec<Vertex> {
        if depth == 0 {
            return vec![];
        }

        let mut ray = ray;
        let mut prev: Vertex = match kind {
            WalkKind::Camera => Vertex::new(
                VertexKind::Camera,
                ray.origin,
                normal,
                beta,
                EPS,
                EPS,
                1.0,
                0.0,
            ),
            WalkKind::Light => Vertex::new(
                VertexKind::Light,
                ray.origin,
                normal,
                beta,
                EPS,
                EPS,
                PI,
                1.0,
            ),
        };
        let mut result: Vec<Vertex> = vec![];

        while result.len() < depth as usize {
            if result.len() > MIN_DEPTH as usize {
                let q = f64::min(1.0, beta.norm() / prev.pdf_fwd);
                if random() > q {
                    break;
                }
            }
            match self.scene.hit(ray, 0.0001..f64::INFINITY) {
                None => {
                    beta = beta.component_mul(&self.scene.background);
                    prev.beta = beta;
                    prev.pdf_rev = 0.0;
                    result.push(prev);
                    prev = Vertex::new(
                        VertexKind::Surface,
                        Point::origin(),
                        Vector::x_axis(),
                        beta,
                        EPS,
                        EPS,
                        1.0,
                        1.0,
                    );
                    break;
                }
                Some(hit) => {
                    let emit = hit.material.emitted(&hit);
                    if emit.norm_squared() > 0.0 {
                        beta = beta.component_mul(&emit);
                        prev.pdf_rev = 0.0;
                        result.push(prev);
                        prev = Vertex::new(
                            VertexKind::Light,
                            hit.point,
                            hit.normal,
                            beta,
                            EPS,
                            EPS,
                            1.0,
                            1.0,
                        );
                        break;
                    }
                    if let Some(scatter) = hit.material.scatter(&ray, &hit) {
                        if let Some(specular) = scatter.specular {
                            ray = specular;
                            prev.pdf_rev = 0.0;
                            result.push(prev);
                            // TODO: Fix this.
                            prev = Vertex::new(
                                VertexKind::Surface,
                                hit.point,
                                hit.normal,
                                beta,
                                EPS,
                                EPS,
                                1.0,
                                1.0,
                            );
                            continue;
                        }
                        if let Some(scatter_pdf) = scatter.pdf {
                            let scatter_ray = Ray {
                                origin: hit.point,
                                dir: scatter_pdf.gen(),
                            };
                            let g_fwd =
                                self.g(hit.point, hit.normal, prev.point);
                            let pdf_fwd = scatter_pdf.value(scatter_ray.dir);
                            let p_i = pdf_fwd * g_fwd;
                            prev.pdf_rev =
                                scatter_pdf.value(prev.point - hit.point);
                            let vcm = pdf_fwd.recip();
                            let g_prev =
                                self.g(prev.point, prev.normal, hit.point);
                            prev.g_rev =
                                scatter_pdf.value(prev.point - hit.point);
                            let vc = match result.last() {
                                Some(v) => {
                                    g_prev / p_i
                                        * (prev.vcm + v.pdf_rev * prev.vc)
                                }
                                None => g_prev / p_i * prev.vcm,
                            };
                            beta = beta.component_mul(&scatter.attenuation);
                            if result.is_empty() {
                                match kind {
                                    WalkKind::Camera => {
                                        prev.vcm = 250.0 / p_i.recip();
                                        prev.vc = 0.0;
                                    }
                                    WalkKind::Light => {
                                        prev.vcm = p_i.recip();
                                        prev.vc =
                                            prev.g_rev / prev.pdf_fwd / p_i;
                                    }
                                }
                            }
                            result.push(prev);
                            prev = Vertex::new(
                                VertexKind::Surface,
                                hit.point,
                                hit.normal,
                                beta,
                                pdf_fwd,
                                g_fwd,
                                vcm,
                                vc,
                            );
                            ray = scatter_ray;
                            continue;
                        }
                    }
                }
            }
        }

        result.push(prev);
        result
    }

    fn test_visibility(&self, v1: &Vertex, v2: &Vertex) -> bool {
        let (p1, p2) =
            (v1.point + EPS * *v1.normal, v2.point + EPS * *v2.normal);
        let ray = Ray {
            origin: p1,
            dir: p2 - p1,
        };
        let tmin = EPS;
        let tmax = ray.dir.norm() - EPS;
        self.scene.hit(ray, tmin..tmax).is_none()
    }

    /// g_i(y) term in the light transport equation.
    /// g_i(y) = cos(theta) / || d ||^2
    /// where theta is the angle between surface normal at v1 and the segment
    /// connecting the vertices.
    /// Forward g_i is calculated by providing vertex i and i-1.
    /// Reverse g_i is calculated by providing vertex i and i+1.
    fn g(&self, p1: Point, n: Unit<Vector>, p2: Point) -> f64 {
        let d = p2 - p1;
        let d_normalized = d.normalize();
        let cos_theta = n.dot(&d_normalized);
        cos_theta / d.norm_squared()
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

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum VertexKind {
    Camera,
    Light,
    Surface,
}

#[derive(Debug, Clone)]
struct Vertex {
    pub kind: VertexKind,
    pub normal: Unit<Vector>,
    pub point: Point,
    pub beta: Color,
    pub pdf_fwd: f64,
    pub g_fwd: f64,
    pub pdf_rev: f64,
    pub g_rev: f64,
    pub vcm: f64,
    pub vc: f64,
}

impl Vertex {
    pub fn new(
        kind: VertexKind,
        point: Point,
        normal: Unit<Vector>,
        beta: Color,
        pdf_fwd: f64,
        g_fwd: f64,
        vcm: f64,
        vc: f64,
    ) -> Vertex {
        Vertex {
            kind,
            normal,
            point,
            beta,
            pdf_fwd,
            g_fwd,
            pdf_rev: 0.0,
            g_rev: 0.0,
            vcm,
            vc,
        }
    }
}
