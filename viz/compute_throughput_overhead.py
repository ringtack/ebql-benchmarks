import sys

import pandas as pd

# Read and process each CSV file
if len(sys.argv) != 4:
    print("Usage: python3 script_name.py file1.csv file2.csv file3.csv")
    sys.exit(1)

def read_and_process_csv(file_path):
    # Load CSV file
    data = pd.read_csv(file_path)
    # Convert columns to numeric, coercing errors to NaN and drop rows with NaN
    data['reads'] = pd.to_numeric(data['reads'], errors='coerce')
    data['second'] = pd.to_numeric(data['second'], errors='coerce')
    data.dropna(subset=['reads', 'second'], inplace=True)
    # Group by 'second' and calculate the average reads
    return data.groupby('second')['reads'].mean()

# Read and process each CSV file using command line arguments
baseline = read_and_process_csv(sys.argv[1])
with_ebql_probe = read_and_process_csv(sys.argv[2])
with_opt_probe = read_and_process_csv(sys.argv[3])

bsum = baseline.mean()
esum = with_ebql_probe.mean()
osum = with_opt_probe.mean()

print(f"{bsum}, {esum}, {osum}")
print(f"{esum / bsum:0.3f}, {osum / bsum:0.3f}")