use crate::color::Color;
use crate::geometry::*;

#[derive(Debug)]
pub enum Texture {
    Solid(Color),
}

impl Texture {
    pub fn value(&self, (_u, _v): (f64, f64), _point: Point) -> Color {
        match *self {
            Texture::Solid(c) => c,
        }
    }
}
