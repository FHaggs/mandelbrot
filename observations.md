## Simple rayon version: Commit: 730fc27ed537a18096686fac640542be7b44823d
Using perf, I saw lots of addsd and divsd instructions. I will try to make this scalar instructions into SIMD like addpd.

Rust (and LLVM) defaults to scalar math unless:

    The code uses explicit SIMD types, like packed_simd or std::simd

    The loop is auto-vectorizable and the compiler can prove it’s safe

    You enable optimizations:
      RUSTFLAGS="-C target-cpu=native" cargo build --release
