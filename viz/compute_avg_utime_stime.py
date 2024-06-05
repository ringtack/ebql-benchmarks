import statistics
import pandas as pd
import sys

# Function to parse and compute statistics
def compute_statistics(file_path):
    utimes = []
    stimes = []

    data = pd.read_csv(file_path, header=None)
    # print(data)
    utimes = data.iloc[:, 0].astype(int)
    stimes = data.iloc[:, 1].astype(int)

    # Calculate mean and standard deviation for utime and stime
    utime_mean = statistics.mean(utimes)
    utime_std_dev = statistics.stdev(utimes)
    stime_mean = statistics.mean(stimes)
    stime_std_dev = statistics.stdev(stimes)

    # Print the results
    print(f"Mean of utime: {utime_mean}")
    print(f"Standard deviation of utime: {utime_std_dev}")
    print(f"Mean of stime: {stime_mean}")
    print(f"Standard deviation of stime: {stime_std_dev}")

if len(sys.argv) != 2:
    print("Usage: python3 script_name.py path_to_data")
    sys.exit(1)

# Path to your data file
file_path = sys.argv[1]

# Call the function with the path to your file
compute_statistics(file_path)
