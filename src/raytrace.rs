use crate::renderer::Buffer;
use crate::scene::*;

use nalgebra::Vector3;
use std::sync::Arc;
use std::sync::RwLock;

pub type Color = Vector3<f64>;

#[derive(Debug, Copy, Clone)]
struct Ray {
    pub origin: Point,
    pub dir: Vector,
}

impl Ray {
    pub fn hit_sphere(&self, sphere: Sphere) -> Option<Point> {
        let oc = self.origin - sphere.center;
        // Solve the quadratic formula.
        let (a, b, c) = (
            self.dir.dot(&self.dir),
            2.0 * oc.dot(&self.dir),
            oc.dot(&oc) - (sphere.radius.powi(2)),
        );
        let discriminant = b.powi(2) - (4.0 * a * c);
        if discriminant < 0.0 {
            None
        } else {
            // b^2 - 4ac > 0 ==> there is at least one root.
            // Subtract discriminant to find the smallest t such that
            // there's an intersection.
            let t = (-b - discriminant.sqrt()) / (2.0 * a);
            Some(self.origin + t * self.dir)
        }
    }
}

fn to_u32(color: Color) -> u32 {
    fn to_256(x: f64) -> u32 {
        (x * 256.0f64) as u8 as u32
    }
    (to_256(color[0]) << 16) | (to_256(color[1]) << 8) | to_256(color[2])
}

pub fn raytrace(
    scene: &Scene,
    buf: Arc<RwLock<Buffer>>,
    im_width: usize,
    im_height: usize,
) {
    let aspect_ratio: f64 = im_width as f64 / im_height as f64;

    let vp_height: f64 = 2.0;
    let vp_width: f64 = aspect_ratio * vp_height;
    let focal_length = 1.0;

    let origin = Point::origin();
    let horiz = Vector::x() * vp_width;
    let vert = Vector::y() * vp_height;
    let lower_left = dbg!(
        origin - (horiz / 2.0) - (vert / 2.0) - (Vector::z() * focal_length)
    );

    for r in 0..im_height {
        for c in 0..im_width {
            let u = c as f64 / (im_width as f64 - 1.0);
            let v = (im_height - r) as f64 / (im_height as f64 - 1.0);
            let dir = (lower_left + (u * horiz) + (v * vert)) - origin;
            let ray = Ray { origin, dir };
            let color = ray_color(scene, ray);
            buf.write().unwrap()[r * im_width + c] = to_u32(color);
        }
    }
}

fn ray_color(scene: &Scene, ray: Ray) -> Color {
    for &sphere in &scene.spheres {
        if let Some(p) = ray.hit_sphere(sphere) {
            let normal = (p - sphere.center).normalize();
            return 0.5
                * Color::new(normal.x + 1.0, normal.y + 1.0, normal.z + 1.0);
        }
    }
    let dir = ray.dir.normalize();
    let t: f64 = 0.5 * (dir.y + 1.0);
    ((1.0 - t) * Color::new(1.0, 1.0, 1.0)) + (t * Color::new(0.5, 0.7, 1.0))
}
