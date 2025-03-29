import numpy as np
import seaborn as sns
import matplotlib.pyplot as plt
import math

def load_values(file_path):
    """Loads float values from a text file, one per line."""
    with open(file_path, "r") as f:
        values = np.array([float(line.strip()) * 100 for line in f if line.strip()])
        values = [math.sqrt(value) for value in values]
        values = [value for value in values if value < 100]
    return values

def plot_distribution(values, output_file="distribution.png"):
    sns.displot(values, bins=50)

    plt.show()

if __name__ == "__main__":
    file_path = "weights.txt"  # Change this to your file
    values = load_values(file_path)
    plot_distribution(values)

