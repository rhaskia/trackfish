import matplotlib.pyplot as plt
import numpy as np

def load_values(file_path):
    with open(file_path, "r") as f:
        values = np.array([float(line.strip()) for line in f if line.strip()])
    return values

x = load_values("spec.txt")

plt.plot(x)
plt.savefig("spec.png", dpi=300, bbox_inches="tight")
plt.close()
