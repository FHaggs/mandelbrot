#!/bin/bash

# Write initial gnuplot script
cat <<EOF > plot_all.gnuplot
set datafile separator comma
set title "Frame Time Comparison"
set xlabel "Frame"
set ylabel "Time (ms)"
set grid
plot \\
EOF

# Dynamically add all data files

find data -name "run_*.csv" | sort | awk '{
    file=$0;
    gsub("data/run_", "", file);
    gsub(".csv", "", file);
    printf "    \"%s\" using 1:2 with lines title \"%s\", \\\n", $0, file
}' >> plot_all.gnuplot

# Remove trailing comma
sed -i '$ s/, *\\$//' plot_all.gnuplot

# Plot
gnuplot -persist plot_all.gnuplot
