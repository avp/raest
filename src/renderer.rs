use crate::config::Config;
use crate::geometry::Scene;
use crate::raytrace;

use minifb::{Window, WindowOptions};
use std::sync::mpsc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

const WIN_WIDTH: usize = 640;
const WIN_HEIGHT: usize = 360;

pub type Buffer = Vec<u32>;

pub fn render(config: Config) {
    let mut window = make_window();
    let buf = Arc::new(RwLock::new(make_buffer()));

    let scene = Scene::random(10);

    {
        let buf = buf.clone();
        thread::spawn(move || {
            raytrace::raytrace(config, &scene, buf, WIN_WIDTH, WIN_HEIGHT);
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
                .update_with_buffer(&*b, WIN_WIDTH, WIN_HEIGHT)
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

fn make_window() -> Window {
    Window::new("Raest", WIN_WIDTH, WIN_HEIGHT, WindowOptions::default())
        .expect("Unable to open window")
}

fn make_buffer() -> Buffer {
    vec![0u32; WIN_WIDTH * WIN_HEIGHT]
}
