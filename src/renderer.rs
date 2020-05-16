use crate::raytrace;
use crate::scene::*;

use minifb::{Window, WindowOptions};
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;

const WIN_WIDTH: usize = 640;
const WIN_HEIGHT: usize = 360;

pub type Buffer = Vec<u32>;

pub fn render() {
    let mut window = make_window();
    let buf = Arc::new(RwLock::new(make_buffer()));

    let scene = Scene {
        spheres: vec![Sphere {
            center: Point::new(0.0, 0.0, -1.0),
            radius: 0.5,
        }],
    };

    {
        let buf = Arc::clone(&buf);
        thread::spawn(move || {
            raytrace::raytrace(&scene, buf, WIN_WIDTH, WIN_HEIGHT);
        });
    }

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        if let Ok(b) = buf.try_read() {
            window
                .update_with_buffer(&*b, WIN_WIDTH, WIN_HEIGHT)
                .unwrap();
        } else {
            // Avoid R/W contention with sleeping, but update the events
            // so we can close the window without blocking.
            window.update();
            thread::sleep(std::time::Duration::from_millis(100));
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
