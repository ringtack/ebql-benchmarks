import sys
import pandas as pd
import matplotlib.pyplot as plt

def read_and_process_csv(file_path):
    # Load CSV file
    data = pd.read_csv(file_path)
    # Convert columns to numeric, coercing errors to NaN and drop rows with NaN
    data['reads'] = pd.to_numeric(data['reads'], errors='coerce')
    data['second'] = pd.to_numeric(data['second'], errors='coerce')
    data.dropna(subset=['reads', 'second'], inplace=True)
    # Group by 'second' and calculate the average reads
    return data.groupby('second')['reads'].mean()

# Read and process each CSV file
if len(sys.argv) != 3:
    print("Usage: python script_name.py path_to_first_file.csv path_to_second_file.csv")
    sys.exit(1)

# Read and process each CSV file using command line arguments
baseline = read_and_process_csv(sys.argv[1])
with_ebql_probe = read_and_process_csv(sys.argv[2])

# Plotting
plt.figure(figsize=(10, 6))
plt.plot(baseline.index, baseline.values, label='Baseline throughput', linewidth=2)
plt.plot(with_ebql_probe.index, with_ebql_probe.values, label='With eBQL probe', linewidth=2)

plt.title('Reads per Second Over Time')
plt.xlabel('Second')
plt.ylabel('Reads (ops/sec)')
plt.grid(True)
plt.legend()  # Add a legend to the plot

# Save the plot to a file
plt.savefig('average_reads_comparison.png', format='png', dpi=300)

# Close the plot figure to free up memory
plt.close()
