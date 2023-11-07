use crate::geometry::Hittable;
use crate::geometry::{Point, Ray, Vector, ONB};
use crate::util::*;
use nalgebra::Unit;
use std::f64::consts::PI;

#[derive(Copy, Clone)]
pub enum PDF<'scene> {
    Cosine(ONB),
    Phong(ONB, Unit<Vector>, u32),
    Hittable(Point, &'scene dyn Hittable),
    Mix(f64, &'scene PDF<'scene>, &'scene PDF<'scene>),
}

impl<'scene> PDF<'scene> {
    pub fn cosine(w: Unit<Vector>) -> PDF<'scene> {
        PDF::Cosine(ONB::from_w(w))
    }

    pub fn phong(
        w: Unit<Vector>,
        outbound: Unit<Vector>,
        n: u32,
    ) -> PDF<'scene> {
        PDF::Phong(ONB::from_w(w), outbound, n)
    }

    pub fn hittable(
        origin: Point,
        hittable: &'scene dyn Hittable,
    ) -> PDF<'scene> {
        PDF::Hittable(origin, hittable)
    }

    pub fn mix(
        bias: f64,
        pdf1: &'scene PDF<'scene>,
        pdf2: &'scene PDF<'scene>,
    ) -> PDF<'scene> {
        PDF::Mix(bias, pdf1, pdf2)
    }

    /// Computes the value of the PDF in the direction of `dir`.
    pub fn value(&self, dir: Vector) -> f64 {
        match self {
            &PDF::Cosine(uvw) => {
                // w \dot dir = |w| |dir| cos(theta).
                // and |w| = |dir| = 1.
                // The total weighted area of the hemisphere is \pi.
                // Therefore, pdf = \cos(theta) / \pi.
                let cos = dir.normalize().dot(&uvw.w);
                f64::max(0.0, cos / PI)
            }
            &PDF::Phong(_uvw, outbound, n) => {
                // http://igorsklyar.com/system/documents/papers/4/fiscourse.comp.pdf
                // pdf(theta) = (n+1)/(2\pi) * cos^n(alpha)
                let cos_theta_s = dir.normalize().dot(&outbound);
                let sin_theta_s = (1.0 - cos_theta_s * cos_theta_s).sqrt();
                (((n + 1) as f64) / (2.0 * PI))
                    * cos_theta_s.powi(n as i32 + 1)
                    * sin_theta_s
            }
            PDF::Hittable(origin, hittable) => hittable.pdf(Ray {
                origin: *origin,
                dir,
            }),
            PDF::Mix(bias, pdf1, pdf2) => {
                bias * pdf1.value(dir) + (1.0 - bias) * pdf2.value(dir)
            }
        }
    }

    /// Generates a random sample from this PDF.
    pub fn gen(&self) -> Vector {
        match self {
            PDF::Cosine(uvw) => uvw.localize(random_cosine_dir()),
            PDF::Phong(uvw, outbound, n) => {
                let r1 = random();
                let r2 = random();
                let cos_theta = r1.powf(((n + 1) as f64).recip());
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();
                let phi = 2.0 * PI * r2;
                let x = cos_theta * phi.sin();
                let y = sin_theta * phi.sin();
                let z = phi.cos();
                ONB::from_w(*outbound).localize(Vector::new(x, y, z))
            }
            PDF::Hittable(origin, hittable) => hittable.random(*origin),
            PDF::Mix(bias, pdf1, pdf2) => {
                if random() < *bias {
                    pdf1.gen()
                } else {
                    pdf2.gen()
                }
            }
        }
    }
}
