#![feature(portable_simd)]
use minifb::{Key, Window, WindowOptions};
use std::fs::File;
use std::io::Write;
use std::time::Instant;

use rayon::prelude::*;
use std::simd::{Mask, Simd, prelude::SimdPartialOrd};

type PaletteArray = [u32; (MAX_ITER + 1) as usize];

const WIDTH: usize = 1280;
const HEIGHT: usize = 720;
const MAX_ITER: u32 = 500;
const LANES: usize = 4;
const CENTER_X: f32 = -0.743639266077433;
const CENTER_Y: f32 = 0.131824786875559;

fn color_from_iter(iter: f32) -> u32 {
    if iter >= MAX_ITER as f32 {
        0x000000
    } else {
        let t = iter / MAX_ITER as f32;
        let r = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f32::consts::PI + 0.0).sin()) * 255.0) as u32;
        let g = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f32::consts::PI + 2.0).sin()) * 255.0) as u32;
        let b = (0.5 * (1.0 + (3.0 * t * 2.0 * std::f32::consts::PI + 4.0).sin()) * 255.0) as u32;
        (r << 16) | (g << 8) | b
    }
}

fn build_palette() -> PaletteArray {
    let mut palette = [0; (MAX_ITER + 1) as usize];
    for i in 0..=MAX_ITER {
        palette[i as usize] = color_from_iter(i as f32);
    }
    palette
}

#[inline]
fn mandelbrot_simd(cx: Simd<f32, LANES>, cy: Simd<f32, LANES>) -> [usize; LANES] {
    let mut zx = Simd::splat(0.0f32);
    let mut zy = Simd::splat(0.0f32);
    let mut iter: Simd<i32, LANES> = Simd::splat(0);
    let mut done = Mask::<i32, LANES>::splat(false);

    for _ in 0..MAX_ITER {
        let zx2 = zx * zx;
        let zy2 = zy * zy;
        let norm: Simd<f32, LANES> = zx2 + zy2;

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

    iter.to_array().map(|i| i as usize)
}

pub fn mandelbrot_image(pixels: &mut [u32], palette: &[u32], scale: f32) {
    let half_w = WIDTH as f32 * 0.5;
    let half_h = HEIGHT as f32 * 0.5;
    let sx = scale / WIDTH as f32;
    let sy = scale / HEIGHT as f32;

    let x_offsets: Simd<f32, LANES> = Simd::from_array([0.0, 1.0, 2.0, 3.0]);

    // Iterate over rows instead of pixels increasing the use of SIMD for each thread
    // and reducing the pressure on thread spawns
    pixels
        .par_chunks_mut(WIDTH)
        .enumerate()
        .for_each(|(y, row)| {
            let cy = Simd::splat(CENTER_Y - half_h * sy) + Simd::splat(y as f32) * Simd::splat(sy);

            for x0 in (0..WIDTH).step_by(LANES) {
                let base_x =
                    Simd::splat(CENTER_X - half_w * sx) + Simd::splat(x0 as f32) * Simd::splat(sx);

                let cx = base_x + x_offsets * Simd::splat(sx);
                let iters: [usize; LANES] = mandelbrot_simd(cx, cy);
                for lane in 0..LANES {
                    let x = x0 + lane;
                    row[x] = palette[iters[lane]];
                }
            }
        });
}

fn main() {
    let run_name = std::env::args().nth(1).expect("no run name given");

    #[cfg(debug_assertions)]
    let mut file =
        File::create(format!("data/run_{}.csv", run_name)).expect("Unable to create file");
    #[cfg(debug_assertions)]
    writeln!(file, "frame,ms").unwrap();

    let mut window = Window::new(
        "Mandelbrot Zoom - ESC to exit",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )
    .unwrap_or_else(|e| panic!("{}", e));

    window.set_target_fps(30);

    let mut scale = 3.5;
    let mut pixels = [0; WIDTH * HEIGHT];

    // pre-compute palette
    let palette: PaletteArray = build_palette();

    let mut iters = 0;

    while window.is_open() && !window.is_key_down(Key::Escape) && iters < 2000 {
        // Automatic zoom-in animation
        scale *= 0.99;

        #[cfg(debug_assertions)]
        {
            let start = Instant::now();
            mandelbrot_image(&mut pixels, &palette, scale);
            let duration = start.elapsed();
            writeln!(file, "{},{}", iters, duration.as_secs_f64() * 1000.0).unwrap();
        }

        #[cfg(not(debug_assertions))]
        mandelbrot_image(&mut pixels, &palette, scale);

        window.update_with_buffer(&pixels, WIDTH, HEIGHT).unwrap();
        iters += 1;
    }
}
