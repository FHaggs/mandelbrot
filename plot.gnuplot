
set datafile separator comma
set title "Frame Time per Frame"
set xlabel "Frame"
set ylabel "Time (ms)"
set grid
set key off
plot "frame_times.csv" using 1:2 with lines linewidth 2 linecolor rgb "blue"