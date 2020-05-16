use nalgebra::{Point3, Vector3};

pub type Point = Point3<f64>;
pub type Vector = Vector3<f64>;

#[derive(Debug)]
pub struct Scene {
    pub spheres: Vec<Sphere>,
}

#[derive(Debug, Copy, Clone)]
pub struct Sphere {
    pub center: Point,
    pub radius: f64,
}
