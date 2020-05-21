use crate::geometry::*;
use std::f64::consts::PI;

use std::ops::Range;

pub fn random_f64(range: Range<f64>) -> f64 {
    use rand::Rng;
    (rand::thread_rng().gen::<f64>() * (range.end - range.start)) + range.start
}

pub fn random_in_unit_sphere() -> Vector {
    let a = random_f64(0.0..2.0 * PI);
    let z = random_f64(-1.0..1.0);
    let r = (1.0 - z.powi(2)).sqrt();
    Vector::new(r * a.cos(), r * a.sin(), z)
}
