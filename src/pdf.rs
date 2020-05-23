use crate::geometry::Hittable;
use crate::geometry::{Point, Ray, Vector, ONB};
use crate::util::*;
use nalgebra::Unit;
use std::f64::consts::PI;
use std::sync::Arc;

#[derive(Clone)]
pub enum PDF {
    Cosine(ONB),
    Hittable(Point, Arc<dyn Hittable>),
}

impl PDF {
    pub fn cosine(w: Unit<Vector>) -> PDF {
        PDF::Cosine(ONB::from_w(w))
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
        }
    }

    /// Generates a random sample from this PDF.
    pub fn gen(&self) -> Vector {
        match self {
            PDF::Cosine(uvw) => uvw.localize(random_cosine_dir()),
            PDF::Hittable(origin, hittable) => hittable.random(*origin),
        }
    }
}
