use minifb::{Key, Window, WindowOptions};
use num_complex::Complex;
use rayon::prelude::*;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const STARTING_MAX_ITER: u32 = 500;

/// Calculates the Mandelbrot iteration count for a given complex number.
///
/// This function includes several optimizations:
/// 1.  **Interior Checking**: Quickly identifies points within the main cardioid and period-2 bulb.
/// 2.  **Continuous Coloring**: Returns a smooth f64 value for smooth color gradients.
/// 3.  **Periodicity Checking**: Detects if an orbit enters a cycle.
fn mandelbrot(c: Complex<f64>, max_iter: u32, use_periodicity_check: bool) -> f64 {
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
    let mut sample_z = Complex::new(0.0, 0.0);

    for i in 0..max_iter {
        // Check if the point escapes
        if z.norm_sqr() > 4.0 {
            // Optimization 2: Continuous Coloring formula
            // This creates a smooth value instead of an integer, preventing color banding.
            let log_zn: f64 = (z.norm() as f64).ln();
            let nu = (log_zn / 2.0f64.ln()).ln() / 2.0f64.ln();
            return i as f64 + 1.0 - nu;
        }

        z = z * z + c;

        // Optimization 3: Periodicity Checking
        // If the orbit enters a cycle, it's inside the set.
        if use_periodicity_check {
            if z == sample_z {
                return max_iter as f64;
            }
            // Take a sample at fixed intervals. A power-of-two interval is also possible.
            if i % 20 == 0 {
                sample_z = z;
            }
        }
    }

    max_iter as f64
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

fn main() {
    let mut pixels: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new(
        "Mandelbrot Zoom - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(60);

    let center_x = -0.743639266077433;
    let center_y = 0.131824786875559;

    let mut scale = 3.5;
    let mut max_iters = STARTING_MAX_ITER;

    // Flag for adaptive periodicity checking
    let mut use_periodicity_check = true;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // Automatic zoom-in animation
        scale *= 0.99; // Slower zoom for a smoother feel
        // max_iters = (max_iters as f64 * 1.02) as u32; // Increase detail as we zoom
        //max_iters = max_iters + 10;

        let scale_x = scale / WIDTH as f64;
        let aspect_ratio = HEIGHT as f64 / WIDTH as f64;
        let scale_y = (scale * aspect_ratio) / HEIGHT as f64;

        // Process pixels in parallel using Rayon
        pixels
            .par_chunks_mut(WIDTH)
            .enumerate()
            .for_each(|(y, row)| {
                for (x, pixel) in row.iter_mut().enumerate() {
                    let cx = center_x + (x as f64 - WIDTH as f64 / 2.0) * scale_x;
                    let cy = center_y + (y as f64 - HEIGHT as f64 / 2.0) * scale_y;
                    let c = Complex::new(cx, cy);

                    let iter_val = mandelbrot(c, max_iters, use_periodicity_check);

                    if iter_val >= max_iters as f64 {
                        // This uses atomic operations, safe for parallelism
                        // but for a simple counter, a non-atomic approach after the loop is fine too.
                        // Here, we'll just count it later for simplicity.
                    }

                    *pixel = color_from_iter(iter_val, max_iters);
                }
            });

        // Recalculate n_inside after the parallel loop
        let n_inside = pixels.iter().filter(|&&p| p == 0).count();

        // Update the adaptive flag for the next frame
        use_periodicity_check = n_inside > (WIDTH * HEIGHT) / 1024;
        // println!(
        //     "Scale: {:.2e}, Max Iters: {}, Inside Px: {}, Periodicity Check: {}",
        //     scale, max_iters, n_inside, use_periodicity_check
        // );

        window.update_with_buffer(&pixels, WIDTH, HEIGHT).unwrap();
    }
}
