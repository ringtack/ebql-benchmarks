import seaborn as sns
import matplotlib.pyplot as plt
import pandas as pd

# Sample data
data = {
    # "Probe type": ["Baseline", "Hand-optimized", "BeeHouse"],
    # "Throughput (ops/sec)": [847041.285, 833138.934, 820035.098],
    "Probe type": ["Baseline", "Simple", "Bestagon"],
    "Throughput (ops/sec)": [847041.285, 704711.702, 820035.098],
    # "Probe type": ["Baseline", "Simple", "Bestagon"],
    # "Throughput (ops/sec)": [847041.285, 791046.032, 820035.098],
}

# Create DataFrame
df = pd.DataFrame(data)

colors = ["blue", "green", "orange"]

# Create a bar plot
sns.barplot(x="Probe type", y="Throughput (ops/sec)", data=df, palette=colors)

# Enhance the plot with labels and title
plt.xlabel("Probe type")
plt.ylabel("Throughput (ops/sec)")
plt.title("Probe Overheads on Throughput")

plt.tight_layout()

plt.savefig(
    "bar_chart.png"
)  # This saves the plot as a PNG file. You can choose PDF, SVG, or other formats by changing the file extension.
