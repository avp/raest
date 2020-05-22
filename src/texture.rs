use crate::color::Color;
use crate::geometry::*;

#[derive(Debug)]
pub enum Texture {
    Solid(Color),
    Checker(Box<Texture>, Box<Texture>),
}

impl Texture {
    pub fn value(&self, (u, v): (f64, f64), point: Point) -> Color {
        match self {
            Texture::Solid(c) => *c,
            Texture::Checker(odd, even) => {
                let amped = 10.0 * point;
                let sines = amped.x.sin() * amped.y.sin() * amped.z.sin();
                if sines < 0.0 {
                    odd.value((u, v), point)
                } else {
                    even.value((u, v), point)
                }
            }
        }
    }
}
