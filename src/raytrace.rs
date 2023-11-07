#[allow(unused_imports)]
use crate::bdpt::BDPT;
use crate::camera::Camera;
use crate::color::Color;
use crate::config::Config;
use crate::geometry::*;
use crate::renderer::Buffer;
#[allow(unused_imports)]
use crate::udpt::UDPT;
use crate::util::*;
use crossbeam::thread;
use image::{ImageBuffer, ImageFormat, Rgb};
use std::ops::Range;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

pub trait Tracer {
    fn sample(&mut self, ray: Ray, debug: bool) -> Color;
}

fn raytrace_rows(
    config: &Config,
    scene: &Scene,
    camera: &Camera,
    buf: Arc<RwLock<Buffer>>,
    rows: Range<usize>,
) {
    // Buffer the rows before writing them to `buf` to avoid too much contention
    // on the lock.
    let mut row_backlog: Vec<(usize, Vec<u32>)> = vec![];
    let mut tracer = UDPT::new(scene);
    // let mut tracer = BDPT::new(scene, config.samples);
    for r in rows {
        let mut row: Vec<u32> = vec![0; config.width];
        for (c, result) in row.iter_mut().enumerate() {
            let mut color_sum = Color::zeros();
            for _ in 0..config.samples {
                let u = (c as f64 + random()) / (config.width as f64 - 1.0);
                let v = ((config.height - r) as f64 + random())
                    / (config.height as f64 - 1.0);
                let ray = camera.get_ray(u, v);
                let color = tracer.sample(ray, false);
                color_sum += color;
            }
            *result = write_color(config, color_sum);
        }
        row_backlog.push((r, row));
        if let Ok(mut b) = buf.try_write() {
            for (r, row) in &row_backlog {
                b[r * config.width..(r + 1) * config.width]
                    .clone_from_slice(&row);
            }
            row_backlog.clear();
        }
    }
    let mut b = buf.write().unwrap();
    for (r, row) in &row_backlog {
        b[r * config.width..(r + 1) * config.width].clone_from_slice(&row);
    }
}

pub fn raytrace<'scene>(
    config: Arc<Config>,
    scene: &'scene Scene,
    camera: &'scene Camera,
    buf: Arc<RwLock<Buffer>>,
) {
    let rows_per = (config.height / config.threads) + 1;

    let start = Instant::now();
    thread::scope(|scope| {
        let config = &config;
        for i in 0..config.threads {
            let buf = buf.clone();
            let start = i * rows_per;
            let end = usize::min(config.height, (i + 1) * rows_per);
            scope.spawn(move |_| {
                raytrace_rows(config, scene, camera, buf, start..end);
            });
        }
    })
    .unwrap();
    let elapsed = start.elapsed();

    println!("Render time: {} ms", elapsed.as_millis());

    if let Some(output) = &config.output {
        let buf = buf.read().unwrap();
        let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_fn(
            config.width as u32,
            config.height as u32,
            |c, r| -> Rgb<u8> {
                let pix = buf[(r * config.width as u32 + c) as usize];
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

fn write_color(config: &Config, color: Color) -> u32 {
    let correct = |mut x: f64| -> u32 {
        if x.is_nan() {
            x = 0.0;
        }
        let scale = 1.0 / config.samples as f64;
        const GAMMA: f64 = 2.0;
        let x2 = (scale * x).powf(1.0 / GAMMA).clamp(0.0, 0.999);
        (x2 * 255.0f64) as u8 as u32
    };
    (correct(color[0]) << 16) | (correct(color[1]) << 8) | correct(color[2])
}
