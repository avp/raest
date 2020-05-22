use crate::color::Color;
use crate::config::Config;
use crate::geometry::*;
use crate::renderer::Buffer;
use crate::util::*;
use crossbeam::thread;
use image::{ImageBuffer, ImageFormat, Rgb};
use std::ops::Range;
use std::sync::Arc;
use std::sync::RwLock;

pub struct Camera {
    pub origin: Point,
    pub lower_left: Point,
    pub horiz: Vector,
    pub vert: Vector,
    u: Vector,
    v: Vector,
    lens_radius: f64,
}

impl Camera {
    fn new(
        from: Point,
        at: Point,
        up: Vector,
        vfov: f64,
        aspect_ratio: f64,
        aperture: f64,
        focus_dist: f64,
    ) -> Camera {
        let theta = vfov.to_radians();
        let h = (theta / 2.0).tan();
        let vp_height: f64 = 2.0 * h;
        let vp_width: f64 = aspect_ratio * vp_height;

        let origin = from;

        // Points from target position to camera.
        let w = (from - at).normalize();
        // Horizontal axis of the camera plane.
        let u = up.cross(&w).normalize();
        // Projects the up vector onto the plane normal to the w vector.
        let v = w.cross(&u);

        // Move the viewport focus_dist away from the camera origin
        // to allow simulating DoF.
        let horiz = focus_dist * u * vp_width;
        let vert = focus_dist * v * vp_height;

        let lower_left = origin - (horiz / 2.0) - (vert / 2.0) - focus_dist * w;

        Camera {
            origin,
            lower_left,
            horiz,
            vert,
            u,
            v,
            lens_radius: aperture / 2.0,
        }
    }

    pub fn get_ray(&self, u: f64, v: f64) -> Ray {
        let rd = self.lens_radius * random_in_unit_disc();
        // let rd = Vector::zeros();
        let offset: Vector = self.u * rd.x + self.v * rd.y;
        let dir = (self.lower_left + (u * self.horiz) + (v * self.vert))
            - self.origin
            - offset;
        Ray {
            origin: self.origin + offset,
            dir,
        }
    }
}

const MAX_DEPTH: u32 = 25;

fn raytrace_rows(
    config: &Config,
    scene: &Scene,
    camera: &Camera,
    buf: Arc<RwLock<Buffer>>,
    im_width: usize,
    im_height: usize,
    rows: Range<usize>,
) {
    let mut row = vec![0; im_width];
    for r in rows {
        for (c, result) in row.iter_mut().enumerate() {
            let mut color_sum = Color::zeros();
            for _ in 0..config.samples {
                let u =
                    (c as f64 + random_f64(0.0..1.0)) / (im_width as f64 - 1.0);
                let v = ((im_height - r) as f64 + random_f64(0.0..1.0))
                    / (im_height as f64 - 1.0);
                let ray = camera.get_ray(u, v);
                color_sum += ray_color(scene, ray, 0);
            }
            *result = write_color(config, color_sum);
        }
        {
            let mut b = buf.write().unwrap();
            b[r * im_width..(r + 1) * im_width].clone_from_slice(&row);
        }
    }
}

pub fn raytrace<'a>(
    config: Config,
    scene: &'a Scene,
    buf: Arc<RwLock<Buffer>>,
    im_width: usize,
    im_height: usize,
) {
    let aspect_ratio: f64 = im_width as f64 / im_height as f64;
    let from = Point::new(13.0, 2.0, 3.0);
    let at = Point::new(0.0, 0.0, 0.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let dist = 10.0;
    let camera = &Camera::new(from, at, up, 20.0, aspect_ratio, 0.1, dist);

    let rows_per = (im_height / config.threads) + 1;
    thread::scope(|scope| {
        let config = &config;
        for i in 0..config.threads {
            let buf = buf.clone();
            let start = i * rows_per;
            let end = usize::min(im_height, (i + 1) * rows_per);
            scope.spawn(move |_| {
                raytrace_rows(
                    config,
                    scene,
                    camera,
                    buf,
                    im_width,
                    im_height,
                    start..end,
                );
            });
        }
    })
    .unwrap();

    if let Some(output) = config.output {
        let buf = buf.read().unwrap();
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(
            im_width as u32,
            im_height as u32,
            |c, r| -> Rgb<u8> {
                let pix = buf[(r * im_width as u32 + c) as usize];
                let red = ((pix >> 16) & 0xff) as u8;
                let green = ((pix >> 8) & 0xff) as u8;
                let blue = (pix & 0xff) as u8;
                Rgb([red, green, blue])
            },
        );
        img.save_with_format(&output, ImageFormat::Png).unwrap();
        println!("Output saved as: {}", output.display());
    }
}

fn ray_color(scene: &Scene, ray: Ray, depth: u32) -> Color {
    if depth > MAX_DEPTH {
        return Color::zeros();
    }
    match scene.hit(ray, 0.0001..f64::INFINITY) {
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

fn write_color(config: &Config, color: Color) -> u32 {
    let correct = |x: f64| -> u32 {
        let scale = 1.0 / config.samples as f64;
        const GAMMA: f64 = 2.0;
        let x2 = (scale * x).powf(1.0 / GAMMA).clamp(0.0, 0.9999);
        (x2 * 256.0f64) as u8 as u32
    };
    (correct(color[0]) << 16) | (correct(color[1]) << 8) | correct(color[2])
}
