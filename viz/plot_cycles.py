import sys
import pandas as pd
import matplotlib.pyplot as plt

def plot_stacked_bar_chart(csv_file):
    # Load the CSV file into a DataFrame
    df = pd.read_csv(csv_file, header=None)

    # Transpose the DataFrame to make each row a category in the bar chart
    df_transposed = df

    # Define a list of colors for the bars
    # colors = ['red', 'blue', 'green', 'purple', 'orange']

    # Define labels for each column; these should be meaningful based on your data
    column_labels = ['RocksDB utime', 'RocksDB stime', 'BPF subsystem', 'BPF probe', 'BPF process utime', 'BPF process stime']

    plt.figure(figsize=(15, 6))
    # Plotting
    ax = df_transposed.plot(kind='bar', stacked=True, width=0.8)#, color=colors)
    ax.set_title('Total CPU Cycles Across RocksDB + eBPF')
    # ax.set_xlabel('Program')
    ax.set_ylabel('Clock Ticks')

    # Define labels for each bar
    bar_labels = ['Baseline RocksDB', 'With eBQL probe', 'With standard probe']
    # Set custom x-axis labels
    ax.set_xticklabels(bar_labels, rotation=0)

    # Adding a legend with descriptive labels
    plt.legend(title='Legend', labels=column_labels)
    plt.tight_layout()
    plt.show()

# Example usage
plot_stacked_bar_chart(sys.argv[1])
# Save the plot to a file or show it
plt.savefig('b2-cycles.png', format='png', dpi=300)
plt.close()