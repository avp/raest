use crate::color::Color;
use crate::geometry::*;
use crate::renderer::Buffer;
use crate::util::*;

use std::sync::Arc;
use std::sync::RwLock;

struct Camera {
    pub origin: Point,
    pub lower_left: Point,
    pub horiz: Vector,
    pub vert: Vector,
}

impl Camera {
    fn new(aspect_ratio: f64) -> Camera {
        let vp_height: f64 = 2.0;
        let vp_width: f64 = aspect_ratio * vp_height;
        let focal_length = 1.0;

        let origin = Point::origin();
        let horiz = Vector::x() * vp_width;
        let vert = Vector::y() * vp_height;
        let lower_left = origin
            - (horiz / 2.0)
            - (vert / 2.0)
            - (Vector::z() * focal_length);

        Camera {
            origin,
            lower_left,
            horiz,
            vert,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let dir = (self.lower_left + (u * self.horiz) + (v * self.vert))
            - self.origin;
        Ray {
            origin: self.origin,
            dir,
        }
    }
}

const NUM_SAMPLES: u32 = 50;
const MAX_DEPTH: u32 = 25;

pub fn raytrace(
    scene: &Scene,
    buf: Arc<RwLock<Buffer>>,
    im_width: usize,
    im_height: usize,
) {
    let aspect_ratio: f64 = im_width as f64 / im_height as f64;
    let camera = Camera::new(aspect_ratio);

    let mut row = vec![Color::new(0.0, 0.0, 0.0); im_width];
    for r in 0..im_height {
        row.clear();
        row.resize(im_width, Color::new(0.0, 0.0, 0.0));
        for c in 0..im_width {
            for _ in 0..NUM_SAMPLES {
                let u =
                    (c as f64 + random_f64(0.0..1.0)) / (im_width as f64 - 1.0);
                let v = ((im_height - r) as f64 + random_f64(0.0..1.0))
                    / (im_height as f64 - 1.0);
                let ray = camera.get_ray(u, v);
                let color = ray_color(scene, ray, 0);
                row[c] += color;
            }
        }

        let mut b = buf.write().unwrap();
        for c in 0..im_width {
            b[r * im_width + c] = write_color(row[c]);
        }
    }
}

fn ray_color(scene: &Scene, ray: Ray, depth: u32) -> Color {
    if depth > MAX_DEPTH {
        return Color::zeros();
    }
    match scene.hit(ray, 0.0001..) {
        Some(hit) => {
            let (outbound, attenuation) = hit.material.scatter(&ray, &hit);
            attenuation.component_mul(&ray_color(scene, outbound, depth + 1))
        }
        None => {
            let dir = ray.dir.normalize();
            let t: f64 = 0.5 * (dir.y + 1.0);
            ((1.0 - t) * Color::new(1.0, 1.0, 1.0))
                + (t * Color::new(0.5, 0.7, 1.0))
        }
    }
}

fn write_color(color: Color) -> u32 {
    fn correct(x: f64) -> u32 {
        let scale = 1.0 / NUM_SAMPLES as f64;
        const GAMMA: f64 = 2.0;
        let x2 = (scale * x).powf(1.0 / GAMMA).clamp(0.0, 0.9999);
        (x2 * 256.0f64) as u8 as u32
    }
    (correct(color[0]) << 16) | (correct(color[1]) << 8) | correct(color[2])
}
