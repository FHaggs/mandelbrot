## Simple rayon version: Commit: 730fc27ed537a18096686fac640542be7b44823d
Using perf, I saw lots of addsd and divsd instructions. I will try to make this scalar instructions into SIMD like addpd.

Rust (and LLVM) defaults to scalar math unless:

    The code uses explicit SIMD types, like packed_simd or std::simd

    The loop is auto-vectorizable and the compiler can prove it’s safe

    You enable optimizations:
      RUSTFLAGS="-C target-cpu=native" cargo build --release

This optimizations are not worth it for simd
    // Optimations for early frames
    let zr = cx;
    let zi = cy;
    let zr_plus_1 = zr + Simd::splat(1.0);
    // Period-2 bulb: (zr + 1)^2 + zi^2 < 1/16
    let bulb_check = (zr_plus_1 * zr_plus_1 + zi * zi).simd_lt(Simd::splat(1.0 / 16.0));
    // Main cardioid: q * (q + (zr - 0.25)) < 0.25 * zi^2
    let zr_minus_025 = zr - Simd::splat(0.25);
    let q = zr_minus_025 * zr_minus_025 + zi * zi;
    let cardioid_check = (q * (q + zr_minus_025)).simd_lt(Simd::splat(0.25) * zi * zi);

    let inside = bulb_check | cardioid_check;
    let mut iter = inside.select(Simd::splat(max_iter as i64), Simd::splat(0));
