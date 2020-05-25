use crate::geometry::Hittable;
use crate::geometry::{Point, Ray, Vector, ONB};
use crate::util::*;
use nalgebra::Unit;
use std::f64::consts::PI;

#[derive(Copy, Clone)]
pub enum PDF<'scene> {
    Cosine(ONB),
    Hittable(Point, &'scene dyn Hittable),
    Mix(f64, &'scene PDF<'scene>, &'scene PDF<'scene>),
}

impl<'scene> PDF<'scene> {
    pub fn cosine(w: Unit<Vector>) -> PDF<'scene> {
        PDF::Cosine(ONB::from_w(w))
    }

    pub fn hittable(
        origin: Point,
        hittable: &'scene dyn Hittable,
    ) -> PDF<'scene> {
        PDF::Hittable(origin, hittable)
    }

    pub fn mix(bias: f64, pdf1: &'scene PDF, pdf2: &'scene PDF) -> PDF<'scene> {
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
            PDF::Hittable(origin, hittable) => hittable.random(*origin),
            PDF::Mix(bias, pdf1, pdf2) => {
                if random_f64(0.0..1.0) < *bias {
                    pdf1.gen()
                } else {
                    pdf2.gen()
                }
            }
        }
    }
}
