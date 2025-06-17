use minifb::{Key, Window, WindowOptions};
use num_complex::Complex;
use rayon::prelude::*;
const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const STARTING_MAX_ITER: u32 = 500;

fn mandelbrot(c: Complex<f64>, max_iter: u32) -> f64 {
    // Optimization 1: Interior Checking for the main cardioid and period-2 bulb.
    let zr = c.re;
    let zi = c.im;
    if (zr + 1.0).powi(2) + zi.powi(2) < 1.0 / 16.0 {
        return max_iter as f64;
    }
    let q = (zr - 0.25).powi(2) + zi.powi(2);
    if q * (q + (zr - 0.25)) < 0.25 * zi.powi(2) {
        return max_iter as f64;
    }

    let mut z = Complex::new(0.0, 0.0);
    let mut z_sample = Complex::new(0.0, 0.0);
    for i in 0..max_iter {
        if z.norm_sqr() > 4.0 {
            return smooth(i, z);
        }
        z = z * z + c;
        if i % 20 == 0 {
            if z_sample == z {
                smooth(i, z);
            }
            z_sample = z;
        }
    }
    return max_iter as f64;
}

fn smooth(x: u32, z: Complex<f64>) -> f64 {
    let log_zn: f64 = (z.norm() as f64).ln();
    let nu = (log_zn / 2.0f64.ln()).ln() / 2.0f64.ln();
    return x as f64 + 1.0 - nu;
}

// An improved function to convert a smooth iteration value to a vibrant RGB color.
fn color_from_iter(iter: f64, max_iter: u32) -> u32 {
    if iter >= max_iter as f64 {
        0x000000 // Black for points inside the set
    } else {
        // Use sine waves for smooth, vibrant color transitions
        let t = iter / max_iter as f64;
        let r = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f64::consts::PI + 0.0).sin()) * 255.0) as u32;
        let g = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f64::consts::PI + 2.0).sin()) * 255.0) as u32;
        let b = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f64::consts::PI + 4.0).sin()) * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }
}
fn maldelbrot_image(
    max_x: usize,
    max_y: usize,
    center_x: f64,
    center_y: f64,
    scale: f64,
    max_iter: u32,
) -> Vec<u32> {
    let mut pixels: Vec<u32> = vec![0; max_x * max_y];
    let scale_x = scale / max_x as f64;
    let scale_y = (scale * HEIGHT as f64 / WIDTH as f64) / HEIGHT as f64;
    pixels.par_iter_mut().enumerate().for_each(|(i, pixel)| {
        let x = i % max_x;
        let y = i / max_x;
        let cx = center_x + (x as f64 - max_x as f64 / 2.0) * scale_x;
        let cy = center_y + (y as f64 - max_y as f64 / 2.0) * scale_y;

        let c = Complex::new(cx, cy);
        let iter = mandelbrot(c, max_iter);
        let color = color_from_iter(iter, max_iter);
        *pixel = color;
    });

    pixels
}

fn main() {
    let mut window = Window::new(
        "Mandelbrot Zoom - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(30);

    let center_x = -0.743639266077433;
    let center_y = 0.131824786875559;

    let mut scale = 3.5;
    // let max_iters = STARTING_MAX_ITER;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Automatic zoom-in animation
        scale *= 0.99;

        let pixels = maldelbrot_image(WIDTH, HEIGHT, center_x, center_y, scale, STARTING_MAX_ITER);

        window.update_with_buffer(&pixels, WIDTH, HEIGHT).unwrap();
    }
}
