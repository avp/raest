#![feature(clamp)]

use std::sync::Arc;
use structopt::StructOpt;

mod camera;
mod config;
mod geometry;
mod material;
mod pdf;
mod raytrace;
mod renderer;
mod texture;
mod util;

mod color {
    use nalgebra::Vector3;
    pub type Color = Vector3<f64>;
}

fn main() {
    let config = config::Config::from_args();
    renderer::render(Arc::new(config));
}
