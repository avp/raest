use crate::geometry::*;
use crate::renderer::Buffer;

use nalgebra::Vector3;
use std::sync::Arc;
use std::sync::RwLock;

pub type Color = Vector3<f64>;

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

const NUM_SAMPLES: u32 = 200;

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
                let u = (c as f64 + random_fuzz()) / (im_width as f64 - 1.0);
                let v = ((im_height - r) as f64 + random_fuzz())
                    / (im_height as f64 - 1.0);
                let ray = camera.get_ray(u, v);
                let color = ray_color(scene, ray);
                row[c] += color;
            }
            row[c] /= NUM_SAMPLES as f64;
        }

        let mut b = buf.write().unwrap();
        for c in 0..im_width {
            b[r * im_width + c] = to_u32(row[c]);
        }
    }
}

fn ray_color(scene: &Scene, ray: Ray) -> Color {
    match scene.hit(ray, 0.0..) {
        Some(hit) => 0.5 * (hit.normal + Vector::new(1.0, 1.0, 1.0)),
        None => {
            let dir = ray.dir.normalize();
            let t: f64 = 0.5 * (dir.y + 1.0);
            ((1.0 - t) * Color::new(1.0, 1.0, 1.0))
                + (t * Color::new(0.5, 0.7, 1.0))
        }
    }
}

fn to_u32(color: Color) -> u32 {
    fn to_256(x: f64) -> u32 {
        (x * 256.0f64) as u8 as u32
    }
    (to_256(color[0]) << 16) | (to_256(color[1]) << 8) | to_256(color[2])
}

fn random_fuzz() -> f64 {
    use rand::Rng;
    rand::thread_rng().gen::<f64>()
}
