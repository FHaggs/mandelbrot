#![feature(portable_simd)]
use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::Write;
use std::time::Instant;

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const STARTING_MAX_ITER: u32 = 500;

use rayon::prelude::*;
use std::simd::prelude::SimdPartialOrd;
use std::simd::{Mask, Simd};

const LANES: usize = 4;

fn mandelbrot_simd(cx: Simd<f64, LANES>, cy: Simd<f64, LANES>, max_iter: u32) -> [f64; LANES] {
    let mut zx = Simd::splat(0.0);
    let mut zy = Simd::splat(0.0);
    let mut iter: Simd<i64, LANES> = Simd::splat(0);
    let mut done = Mask::<i64, LANES>::splat(false);

    for _ in 0..max_iter {
        let zx2 = zx * zx;
        let zy2 = zy * zy;
        let norm: Simd<f64, LANES> = zx2 + zy2;

        let mask = norm.simd_lt(Simd::splat(4.0)) & !done;

        if !mask.any() {
            break;
        }

        let new_zx = zx2 - zy2 + cx;
        let new_zy = Simd::splat(2.0) * zx * zy + cy;

        zx = mask.select(new_zx, zx);
        zy = mask.select(new_zy, zy);
        iter = mask.select(iter + Simd::splat(1), iter);
        done |= !mask;
    }

    let iter_f64: [f64; LANES] = iter.to_array().map(|i| i as f64);
    return iter_f64;
}

fn color_from_iter(iter: f64, max_iter: u32) -> u32 {
    if iter >= max_iter as f64 {
        0x000000
    } else {
        let t = iter / max_iter as f64;
        let r = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f64::consts::PI + 0.0).sin()) * 255.0) as u32;
        let g = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f64::consts::PI + 2.0).sin()) * 255.0) as u32;
        let b = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f64::consts::PI + 4.0).sin()) * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }
}

pub fn mandelbrot_image(
    pixels: &mut [u32],
    width: usize,
    height: usize,
    center_x: f64,
    center_y: f64,
    scale: f64,
    max_iter: u32,
) {
    let scale_x = scale / width as f64;
    let scale_y = (scale * height as f64 / width as f64) / height as f64;

    pixels
        .par_chunks_mut(LANES)
        .enumerate()
        .for_each(|(chunk_idx, chunk)| {
            let base = chunk_idx * LANES;

            // SAFETY: We make sure not to go out of bounds
            let mut cx_array = [0.0; LANES];
            let mut cy_array = [0.0; LANES];

            for i in 0..chunk.len() {
                let pixel_index = base + i;
                let x = pixel_index % width;
                let y = pixel_index / width;

                let cx = center_x + (x as f64 - width as f64 / 2.0) * scale_x;
                let cy = center_y + (y as f64 - height as f64 / 2.0) * scale_y;

                cx_array[i] = cx;
                cy_array[i] = cy;
            }

            let cx = Simd::from_array(cx_array);
            let cy = Simd::from_array(cy_array);
            let results = mandelbrot_simd(cx, cy, max_iter);

            for i in 0..chunk.len() {
                chunk[i] = color_from_iter(results[i], max_iter);
            }
        });
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
    let mut pixels: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let run_name = std::env::args().nth(1).expect("no run name given");
    let mut file =
        File::create(format!("data/run_{}.csv", run_name)).expect("Unable to create file");
    writeln!(file, "frame,ms").unwrap();
    let mut iters = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) && iters < 2000 {
        // Automatic zoom-in animation
        scale *= 0.99;
        let start = Instant::now();
        mandelbrot_image(
            &mut pixels,
            WIDTH,
            HEIGHT,
            center_x,
            center_y,
            scale,
            STARTING_MAX_ITER,
        );
        let duration = start.elapsed();

        writeln!(file, "{},{}", iters, duration.as_secs_f64() * 1000.0).unwrap();

        window.update_with_buffer(&pixels, WIDTH, HEIGHT).unwrap();
        iters += 1;
    }
}
