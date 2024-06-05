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
if len(sys.argv) != 5:
    print("Usage: python script_name.py file1.csv file2.csv file3.csv file4.csv")
    sys.exit(1)

# Read and process each CSV file using command line arguments
# Load data from two CSV files
quantiles1, values1 = load_quantiles(sys.argv[1])
quantiles2, values2 = load_quantiles(sys.argv[2])
quantiles3, values3 = load_quantiles(sys.argv[3])
quantiles4, values4 = load_quantiles(sys.argv[4])

# Plotting
plt.figure(figsize=(12, 6))

plt.plot(quantiles1, values1, marker='o', linestyle='-', label='Baseline')
plt.plot(quantiles2, values2, marker='x', linestyle='--', label='With eBQL probe')
plt.plot(quantiles3, values3, marker='*', linestyle='-.', label='With optimized probe')
plt.plot(quantiles4, values4, marker='+', linestyle=':', label='With simple probe')

plt.title('Comparison of Quantile Latencies')
plt.xlabel('Quantiles')
plt.ylabel('Latency (µs)')
# plt.grid(True)
plt.legend()

# Set x-axis ticks to show actual quantile values but spaced equidistantly
# plt.xticks(quantiles1, [f"{q:.3f}" for q in quantiles1])

# Annotate each point with its quantile value
avg_values = (values1 + values2 + values3) / 3

for q, v in zip(quantiles1, avg_values):
    plt.annotate(f"{q:.3f}", (q, v), textcoords="offset points", xytext=(0,10), ha='center')

# Adjust the x-axis to span from 0 to 1
plt.xlim(0.0, 1.02)

# Save the plot to a file or show it
plt.savefig('b1-quantiles.png', format='png', dpi=300)
plt.close()
