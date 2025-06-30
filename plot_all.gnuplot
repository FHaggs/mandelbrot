set datafile separator comma
set title "Frame Time Comparison"
set xlabel "Frame"
set ylabel "Time (ms)"
set grid
plot \
    "data/run_simd_try.csv" using 1:2 with lines title "simd_try", \
    "data/run_simple_rayon.csv" using 1:2 with lines title "simple_rayon", \
    "data/run_single_thread.csv" using 1:2 with lines title "single_thread"
