use crate::color::Color;
use crate::geometry::*;
use crate::util::*;
use image::{DynamicImage, GenericImageView, Pixel};
use std::sync::Arc;

#[derive(Clone)]
pub enum Texture {
    Solid(Color),
    Checker(Arc<Texture>, Arc<Texture>),
    Image(DynamicImage),
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
            Texture::Image(img) => {
                let u = fclamp(u, 0.0, 1.0);
                let v = 1.0 - fclamp(v, 0.0, 1.0);
                let x =
                    clamp((u * img.width() as f64) as u32, 0, img.width() - 1);
                let y = clamp(
                    (v * img.height() as f64) as u32,
                    0,
                    img.height() - 1,
                );
                let pixel = img.get_pixel(x, y).to_rgb().0;
                Color::new(
                    pixel[0] as f64 / 255.0,
                    pixel[1] as f64 / 255.0,
                    pixel[2] as f64 / 255.0,
                )
            }
        }
    }
}
