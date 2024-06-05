import seaborn as sns
import matplotlib.pyplot as plt
import pandas as pd
import numpy as np
import itertools

# Quantiles - assumed same for simplicity across three datasets
quantiles = [0.01, 0.1, 0.25, 0.5, 0.75, 0.85, 0.9, 0.95, 0.975, 0.99, 0.999]

# Example latency values in microseconds for three different scenarios
lvals1 = np.array(
    [
        0.000001978,
        0.000002764,
        0.000006343,
        0.000008338,
        0.00001174,
        0.000012861,
        0.000013931,
        0.000016826,
        0.000017525,
        0.00001814,
        0.000022437,
    ]
)
lvals1 *= 1000000
lvals2 = np.array(
    [
        0.000001936,
        0.000002722,
        0.000006486,
        0.000008456,
        0.000012064,
        0.000013157,
        0.000014202,
        0.000017309,
        0.000018015,
        0.000018636,
        0.000022969,
    ]
)
lvals2 *= 1000000
lvals3 = np.array(
    [
        0.000001941,
        0.000002727,
        0.00000659,
        0.000008606,
        0.000012291,
        0.000013397,
        0.000014461,
        0.000017639,
        0.000018345,
        0.000018971,
        0.000023329,
    ]
)
lvals3 *= 1000000

# Create a DataFrame for each dataset
df1 = pd.DataFrame(
    {"Latency (µs)": lvals1, "Quantile": quantiles, "Dataset": "Baseline"}
)

df2 = pd.DataFrame(
    {"Latency (µs)": lvals2, "Quantile": quantiles, "Dataset": "Hand-optimized"}
)
df3 = pd.DataFrame(
    {"Latency (µs)": lvals3, "Quantile": quantiles, "Dataset": "BeeHouse"}
)

# Combine all datasets into one DataFrame
df_combined = pd.concat([df1, df2, df3])

# Plot the quantile as a line plot for each dataset
plt.figure(figsize=(10, 6))
sns.lineplot(
    x="Latency (µs)",
    y="Quantile",
    hue="Dataset",
    marker="o",
    data=df_combined,
    palette="tab10",
)

# Adding labels and title for clarity
plt.xlabel("Latency (µs)")
plt.ylabel("Quantile")
plt.title("CDF of Latency Across Different Scenarios")
plt.grid(True)
plt.legend(title="Dataset")  # Adding a title to the legend

plt.tight_layout()

# Saves plot as PNG file
plt.savefig("quantiles.png")
