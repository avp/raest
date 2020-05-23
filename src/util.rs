use crate::geometry::*;
use std::f64::consts::PI;

use nalgebra::Unit;
use std::ops::Range;

pub fn random_f64(range: Range<f64>) -> f64 {
    use rand::Rng;
    rand::thread_rng().gen_range(range.start, range.end)
}

pub fn random_in_unit_disc() -> Vector {
    let theta = random_f64(0.0..2.0 * PI);
    let r = random_f64(0.0..1.0);
    Vector::new(r * theta.cos(), r * theta.sin(), 0.0)
}

pub fn random_unit_vector() -> Vector {
    let a = random_f64(0.0..2.0 * PI);
    let z = random_f64(-1.0..1.0);
    let r = (1.0 - z * z).sqrt();
    Vector::new(r * a.cos(), r * a.sin(), z)
}

pub fn random_in_unit_sphere() -> Vector {
    let a = random_f64(0.0..2.0 * PI);
    let z = random_f64(-1.0..1.0);
    let r = (1.0 - z * z).sqrt();
    Vector::new(r * a.cos(), r * a.sin(), z)
}

pub fn random_cosine_dir() -> Vector {
    let r1 = random_f64(0.0..1.0);
    let r2 = random_f64(0.0..1.0);
    let z = (1.0 - r2).sqrt();

    let phi = 2.0 * PI * r1;
    let r2_sqrt = r2.sqrt();
    let x = phi.cos() * r2_sqrt;
    let y = phi.sin() * r2_sqrt;

    return Vector::new(x, y, z);
}

pub fn reflect(vec: Vector, n: Unit<Vector>) -> Vector {
    vec - (2.0 * vec.dot(&n) * *n)
}

/// Refract vec and surface normal `n` according to the ratio of the IORs
/// `eta`.
pub fn refract(vec: Vector, n: Unit<Vector>, eta: f64) -> Vector {
    let cos_theta = (-vec).dot(&n);
    let parallel = eta * (vec + cos_theta * *n);
    let perp = -((1.0 - parallel.norm_squared()).sqrt() * *n);
    parallel + perp
}

#[cfg(test)]
#[test]
fn tester() {
    println!(
        "{:?}",
        refract(
            Vector::new(1.0, 1.0, 0.0).normalize(),
            -Vector::y_axis(),
            1.0 / 1.33333333
        )
    );
}

pub fn schlick(cosine: f64, ior: f64) -> f64 {
    let r0 = (1.0 - ior) / (1.0 + ior);
    let r0 = r0 * r0;
    let one_minus_cos = 1.0 - cosine;
    r0 + (1.0 - r0)
        * (one_minus_cos
            * one_minus_cos
            * one_minus_cos
            * one_minus_cos
            * one_minus_cos)
}

pub fn clamp<T: Ord>(val: T, min: T, max: T) -> T {
    val.max(min).min(max)
}

pub fn fclamp(val: f64, min: f64, max: f64) -> f64 {
    val.max(min).min(max)
}
