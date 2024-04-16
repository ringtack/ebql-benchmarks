import sys
import numpy as np
import pandas as pd
import matplotlib.pyplot as plt

def load_quantiles(file_path):
    # Load the CSV file
    data = pd.read_csv(file_path, header=None)
    # Extract quantiles and values assuming quantiles are in the first row and values in the second
    quantiles = data.iloc[0, :].astype(float)
    values = data.iloc[1, :].astype(float) * 1e6
    return quantiles, values

# Read and process each CSV file
if len(sys.argv) != 3:
    print("Usage: python script_name.py path_to_first_file.csv path_to_second_file.csv")
    sys.exit(1)

# Read and process each CSV file using command line arguments
# Load data from two CSV files
quantiles1, values1 = load_quantiles(sys.argv[1])
quantiles2, values2 = load_quantiles(sys.argv[2])

# Plotting
plt.figure(figsize=(12, 6))

plt.plot(quantiles1, values1, marker='o', linestyle='-', label='Baseline')
plt.plot(quantiles2, values2, marker='x', linestyle='--', label='With eBQL Probe')

plt.title('Comparison of Quantile Latencies')
plt.xlabel('Quantiles')
plt.ylabel('Latency (µs)')
plt.grid(True)
plt.legend()

# Format x-axis ticks to show quantile values (combined and sorted from both datasets)
all_quantiles = sorted(set(quantiles1).union(set(quantiles2)))
# Set x-axis ticks to show actual quantile values but spaced equidistantly
# plt.xticks(np.arange(len(quantiles1)), [f"{q:.2f}" for q in quantiles1])
plt.xticks(all_quantiles, [f"{q:.3f}" for q in all_quantiles])

# Save the plot to a file or show it
plt.savefig('quantile_comparison.png', format='png', dpi=300)
plt.close()
