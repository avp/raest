use crate::config::Config;
use crate::geometry::Scene;
use crate::raytrace;

use minifb::{Window, WindowOptions};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

pub type Buffer = Vec<u32>;

pub fn render(config: Arc<Config>) {
    let mut window = make_window(&config);
    let buf = Arc::new(RwLock::new(make_buffer(&config)));
    let (scene, camera) = Scene::from_config(&config);

    {
        let buf = buf.clone();
        let config = config.clone();
        thread::spawn(move || {
            raytrace::raytrace(config, &scene, &camera, buf);
        });
    }

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || loop {
        thread::sleep(std::time::Duration::from_millis(500));
        if tx.send(()).is_err() {
            break;
        }
    });

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        thread::sleep(std::time::Duration::from_millis(10));
        if let Ok(b) = buf.try_read() {
            window
                .update_with_buffer(&*b, config.width, config.height)
                .unwrap();
        } else {
            // Avoid R/W contention with sleeping, but update the events
            // so we can close the window without blocking.
            loop {
                window.update();
                if rx.try_recv().is_ok() {
                    break;
                }
            }
        }
    }
}

fn make_window(config: &Config) -> Window {
    Window::new(
        "Raest",
        config.width,
        config.height,
        WindowOptions::default(),
    )
    .expect("Unable to open window")
}

fn make_buffer(config: &Config) -> Buffer {
    vec![0u32; config.width * config.height]
}
