#![feature(clamp)]

use structopt::StructOpt;

mod config;
mod geometry;
mod material;
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
    renderer::render(config);
}
