import os
import os.path as osp
import json
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
import math


folder = osp.dirname(osp.dirname(__file__))
folder = osp.join(folder, ".benchmarks")
subfolders = [i for i in os.listdir(folder) if osp.isdir(osp.join(folder, i))]
folder = osp.join(folder, subfolders[0])

results = []

for fname in os.listdir(folder):
    if not fname.endswith("json"):
        continue
    
    run_name = fname.rstrip("_benchmark.json")
    run_name = int(run_name)
    
    path = osp.join(folder, fname)
    with open(path, "r") as data:
        data = data.read()
    
    data = json.loads(data)
    for entry in data['benchmarks']:
        test_name = entry['fullname']\
            .replace("test_py_benchmark.py::", "")\
            .replace("tests/", "")
        mean = entry['stats']['mean']
        results.append([ run_name, test_name, mean ])
        
results = pd.DataFrame(
    results,
    columns=["run", "test", "average"]
    )
results = results.sort_values("run")

tests = results["test"].unique()

# ============================================================================= 
# Plotting
# ============================================================================= 

cols = 3
rows = math.ceil(len(tests) / cols)

fig, axes = plt.subplots(
    rows,
    cols,
    # figsize=(cols * 5, rows * 4),
    squeeze=True,
)

# Consistent Y-axis across all subplots

for ax, test in zip(axes.flat, tests):
    data = results[results["test"] == test]

    sns.barplot(
        data=data,
        x="run",
        y="average",
        ax=ax,
        width=0.45,       # Thinner bars
        errorbar=None,
    )

    ax.set_title(test)

    # Consistent Y range
    # Y-axis maximum specific to this benchmark
    y_max = data["average"].max() * 1.15
    ax.set_ylim(0, y_max)

    # Remove excessive horizontal padding
    ax.margins(x=0.08)

    ax.tick_params(axis="x", rotation=45)
    ax.grid(axis="y", alpha=0.25)
    ax.set_axisbelow(True)
    
    ax.set_title(test, fontsize=7)
    ax.set_xlabel("", fontsize=7)
    ax.set_ylabel("Mean (seconds)", fontsize=5)

    ax.tick_params(axis="x", labelsize=5, rotation=45)
    ax.tick_params(axis="y", labelsize=5)

# Hide unused subplots
for ax in axes.flat[len(tests):]:
    ax.set_visible(False)

fig.suptitle("Benchmark Results", fontsize=16)
plt.tight_layout()
plt.show()