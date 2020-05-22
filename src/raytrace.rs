use crate::camera::Camera;
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
use std::time::Instant;

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

    // let from = Point::new(13.0, 2.0, 3.0);
    // let at = Point::new(0.0, 0.0, 0.0);
    // let up = Vector::new(0.0, 1.0, 0.0);
    // let dist = 10.0;
    // let camera = &Camera::new(from, at, up, 20.0, aspect_ratio, 0.1, dist);

    let from = Point::new(278.0, 278.0, -800.0);
    let at = Point::new(278.0, 278.0, 0.0);
    let up = Vector::new(0.0, 1.0, 0.0);
    let dist = 10.0;
    let camera = &Camera::new(from, at, up, 40.0, aspect_ratio, 0.0, dist);

    let start = Instant::now();

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

    let elapsed = start.elapsed();
    println!("Render time: {} ms", elapsed.as_millis());

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
            let emit = hit.material.emitted(&hit);
            match hit.material.scatter(&ray, &hit) {
                None => emit,
                Some((outbound, attenuation)) => {
                    emit + attenuation.component_mul(&ray_color(
                        scene,
                        outbound,
                        depth + 1,
                    ))
                }
            }
        }
        None => scene.background,
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
