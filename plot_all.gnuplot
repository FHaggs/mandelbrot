set datafile separator comma
set title "Frame Time Comparison"
set xlabel "Frame"
set ylabel "Time (ms)"
set grid
plot \
    "data/run_gpt_version.csv" using 1:2 with lines title "gpt_version", \
    "data/run_simd_flatten.csv" using 1:2 with lines title "simd_flatten", \
    "data/run_simd_test.csv" using 1:2 with lines title "simd_test", \
    "data/run_simple_rayon.csv" using 1:2 with lines title "simple_rayon"
