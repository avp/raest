#![feature(clamp)]

mod geometry;
mod material;
mod ray;
mod raytrace;
mod renderer;
mod util;

mod color {
    use nalgebra::Vector3;
    pub type Color = Vector3<f64>;
}

fn main() {
    renderer::render();
}
