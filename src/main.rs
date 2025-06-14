use minifb::{Key, Window, WindowOptions};
use num_complex::Complex;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const MAX_ITER: u32 = 100;

fn mandelbrot(c: Complex<f64>, max_iter: u32) -> u32 {
    let mut z = Complex::new(0.0, 0.0);
    for i in 0..max_iter {
        if z.norm_sqr() > 4.0 {
            return i;
        }
        z = z * z + c;
    }
    max_iter
}

// A simple function to convert smooth iteration count to RGB color
fn color_from_iter(iter: f64, max_iter: u32) -> u32 {
    if iter >= max_iter as f64 {
        0x000000 // black for points inside the set
    } else {
        let t = iter / max_iter as f64;
        // Example: simple smooth gradient from blue to red
        let r = (9.0 * (1.0 - t) * t * t * t * 255.0) as u32;
        let g = (15.0 * (1.0 - t) * (1.0 - t) * t * t * 255.0) as u32;
        let b = (8.5 * (1.0 - t) * (1.0 - t) * (1.0 - t) * t * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }
}

fn main() {
    let mut pixels: Vec<u32> = vec![0; WIDTH * HEIGHT];

    let mut window = Window::new(
        "Test - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    // Limit to max ~60 fps update rate
    window.set_target_fps(30);

    let center_x = -0.7436438870371587; // Example interesting zoom center x
    let center_y = 0.13182590420531198; // Example interesting zoom center y

    let mut scale = 3.5;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        scale *= 0.9;
        let scale_x = scale / WIDTH as f64;
        let scale_y = (scale * HEIGHT as f64 / WIDTH as f64) / HEIGHT as f64;

        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let cx = center_x + (x as f64 - WIDTH as f64 / 2.0) * scale_x;
                let cy = center_y + (y as f64 - HEIGHT as f64 / 2.0) * scale_y;
                let c = Complex::new(cx, cy);

                // Mandelbrot iteration
                let iter = mandelbrot(c, MAX_ITER);
                pixels[y * WIDTH + x] = color_from_iter(iter as f64, MAX_ITER);
            }
        }

        window.update_with_buffer(&pixels, WIDTH, HEIGHT).unwrap();
    }
}
