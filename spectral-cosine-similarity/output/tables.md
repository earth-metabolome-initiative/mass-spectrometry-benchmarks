# Benchmark Tables

## Run Scope

- Total spectra in DB: `199032`
- Spectra used in results: `199026`

## Timing by Peak Count (Spectra used: 199026)

Y-axis: `Mean time (µs)`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (matchms) | 1.861e1 ± 3.50e0 (n=280820) | 1.975e1 ± 4.58e0 (n=338350) | 2.217e1 ± 5.90e0 (n=256184) | 2.576e1 ± 6.85e0 (n=96817) | 3.044e1 ± 9.75e0 (n=24006) | 3.639e1 ± 1.09e1 (n=3746) | 4.479e1 ± 7.55e0 (n=77) |
| CosineHungarian (mass-spectrometry-traits) | 1.108e0 ± 9.96e-1 (n=280820) | 1.432e0 ± 1.07e0 (n=338350) | 2.099e0 ± 1.20e0 (n=256184) | 3.337e0 ± 1.46e0 (n=96817) | 6.117e0 ± 4.49e0 (n=24006) | 9.878e0 ± 7.35e0 (n=3746) | 1.465e1 ± 4.05e0 (n=77) |
| CosineHungarian (matchms) | 2.750e1 ± 1.29e1 (n=280820) | 3.185e1 ± 1.71e1 (n=338350) | 4.081e1 ± 2.17e1 (n=256184) | 5.318e1 ± 2.49e1 (n=96817) | 7.193e1 ± 6.09e1 (n=24006) | 8.962e1 ± 8.72e1 (n=3746) | 1.186e2 ± 6.23e1 (n=77) |

### Reference: CosineHungarianMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarianMerged (mass-spectrometry-traits) | 1.031e0 ± 9.25e-1 (n=280820) | 1.327e0 ± 9.98e-1 (n=338350) | 1.938e0 ± 1.13e0 (n=256184) | 3.046e0 ± 1.37e0 (n=96817) | 4.979e0 ± 1.88e0 (n=24006) | 8.322e0 ± 3.09e0 (n=3746) | 1.374e1 ± 3.98e0 (n=77) |
| LinearCosine (mass-spectrometry-traits) | 9.949e-1 ± 9.00e-1 (n=280820) | 1.253e0 ± 9.54e-1 (n=338350) | 1.751e0 ± 1.06e0 (n=256184) | 2.672e0 ± 1.21e0 (n=96817) | 4.227e0 ± 1.39e0 (n=24006) | 6.929e0 ± 1.63e0 (n=3746) | 1.052e1 ± 1.09e0 (n=77) |

### Reference: EntropySimilarityUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 4.519e-1 ± 3.12e-1 (n=280820) | 5.462e-1 ± 3.36e-1 (n=338350) | 7.225e-1 ± 3.96e-1 (n=256184) | 1.017e0 ± 5.26e-1 (n=96817) | 1.270e0 ± 7.52e-1 (n=24006) | 1.449e0 ± 9.82e-1 (n=3746) | 1.861e0 ± 1.13e0 (n=77) |
| EntropySimilarityUnweighted (ms_entropy) | 9.624e0 ± 3.09e0 (n=280820) | 1.035e1 ± 3.30e0 (n=338350) | 1.187e1 ± 3.68e0 (n=256184) | 1.493e1 ± 4.16e0 (n=96817) | 2.054e1 ± 4.57e0 (n=24006) | 2.998e1 ± 5.27e0 (n=3746) | 4.371e1 ± 4.25e0 (n=77) |

### Reference: EntropySimilarityWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.942e-1 ± 4.45e-1 (n=280820) | 9.714e-1 ± 4.74e-1 (n=338350) | 1.279e0 ± 5.57e-1 (n=256184) | 1.693e0 ± 7.06e-1 (n=96817) | 1.960e0 ± 9.61e-1 (n=24006) | 2.187e0 ± 1.25e0 (n=3746) | 2.712e0 ± 1.40e0 (n=77) |
| EntropySimilarityWeighted (ms_entropy) | 1.033e1 ± 3.14e0 (n=280820) | 1.109e1 ± 3.35e0 (n=338350) | 1.269e1 ± 3.71e0 (n=256184) | 1.582e1 ± 4.20e0 (n=96817) | 2.148e1 ± 4.66e0 (n=24006) | 3.108e1 ± 5.38e0 (n=3746) | 4.480e1 ± 4.51e0 (n=77) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosine (mass-spectrometry-traits) | 1.233e0 ± 1.06e0 (n=280820) | 1.590e0 ± 1.14e0 (n=338350) | 2.295e0 ± 1.30e0 (n=256184) | 3.681e0 ± 1.61e0 (n=96817) | 6.828e0 ± 4.70e0 (n=24006) | 1.134e1 ± 8.02e0 (n=3746) | 1.763e1 ± 6.31e0 (n=77) |
| ModifiedCosineHungarian (matchms) | 3.809e1 ± 1.86e1 (n=280820) | 4.284e1 ± 1.99e1 (n=338350) | 5.088e1 ± 2.08e1 (n=256184) | 6.219e1 ± 2.32e1 (n=96817) | 8.169e1 ± 5.12e1 (n=24006) | 1.030e2 ± 6.77e1 (n=3746) | 1.448e2 ± 7.24e1 (n=77) |
| ModifiedGreedyCosine (matchms) | 2.782e1 ± 5.73e0 (n=280820) | 2.932e1 ± 6.21e0 (n=338350) | 3.188e1 ± 6.57e0 (n=256184) | 3.548e1 ± 6.84e0 (n=96817) | 4.078e1 ± 9.32e0 (n=24006) | 4.755e1 ± 1.01e1 (n=3746) | 5.727e1 ± 9.45e0 (n=77) |

### Reference: ModifiedCosineMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosineMerged (mass-spectrometry-traits) | 1.150e0 ± 9.85e-1 (n=280820) | 1.476e0 ± 1.06e0 (n=338350) | 2.117e0 ± 1.22e0 (n=256184) | 3.355e0 ± 1.51e0 (n=96817) | 5.599e0 ± 2.18e0 (n=24006) | 9.594e0 ± 4.26e0 (n=3746) | 1.611e1 ± 5.46e0 (n=77) |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.152e0 ± 9.87e-1 (n=280820) | 1.456e0 ± 1.06e0 (n=338350) | 2.054e0 ± 1.21e0 (n=256184) | 3.195e0 ± 1.47e0 (n=96817) | 5.178e0 ± 1.88e0 (n=24006) | 8.596e0 ± 2.66e0 (n=3746) | 1.366e1 ± 2.69e0 (n=77) |

### Reference: ModifiedLinearEntropyUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.404e-1 ± 3.49e-1 (n=280820) | 6.542e-1 ± 3.85e-1 (n=338350) | 8.678e-1 ± 4.74e-1 (n=256184) | 1.230e0 ± 6.77e-1 (n=96817) | 1.538e0 ± 1.01e0 (n=24006) | 1.764e0 ± 1.37e0 (n=3746) | 2.289e0 ± 1.51e0 (n=77) |

### Reference: ModifiedLinearEntropyWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 8.891e-1 ± 4.82e-1 (n=280820) | 1.085e0 ± 5.22e-1 (n=338350) | 1.431e0 ± 6.32e-1 (n=256184) | 1.913e0 ± 8.52e-1 (n=96817) | 2.235e0 ± 1.21e0 (n=24006) | 2.507e0 ± 1.62e0 (n=3746) | 3.130e0 ± 1.73e0 (n=77) |

## RMSE vs Reference by Peak Count (Spectra used: 199026)

Y-axis: `RMSE`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (matchms) | 1.522e-4 ± 1.50e-3 (n=280820) | 5.380e-6 ± 6.47e-5 (n=338350) | 3.519e-5 ± 3.95e-4 (n=256184) | 1.586e-5 ± 1.12e-4 (n=96817) | 4.234e-6 ± 7.69e-6 (n=24006) | 3.144e-6 ± 6.42e-6 (n=3746) | 1.707e-15 ± 1.85e-15 (n=77) |
| CosineHungarian (mass-spectrometry-traits) | 1.141e-4 ± 1.20e-3 (n=280820) | 4.048e-4 ± 4.88e-3 (n=338350) | 2.681e-4 ± 2.45e-3 (n=256184) | 3.447e-5 ± 1.96e-4 (n=96817) | 3.702e-6 ± 2.14e-5 (n=24006) | 1.358e-15 ± 2.23e-15 (n=3746) | 1.707e-15 ± 1.86e-15 (n=77) |

### Reference: CosineHungarianMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| LinearCosine (mass-spectrometry-traits) | 1.000e-16 ± 4.68e-23 (n=280820) | 1.000e-16 ± 4.79e-23 (n=338350) | 1.000e-16 ± 4.62e-23 (n=256184) | 1.000e-16 ± 2.86e-23 (n=96817) | 1.000e-16 ± 1.27e-23 (n=24006) | 1.000e-16 ± 7.99e-24 (n=3746) | 1.000e-16 (n=77) |

### Reference: EntropySimilarityUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (ms_entropy) | 1.408e-4 ± 1.26e-3 (n=280820) | 5.780e-4 ± 5.45e-3 (n=338350) | 3.286e-4 ± 2.35e-3 (n=256184) | 3.893e-4 ± 1.87e-3 (n=96817) | 1.137e-3 ± 6.96e-3 (n=24006) | 3.589e-4 ± 1.17e-3 (n=3746) | 3.215e-4 ± 5.03e-4 (n=77) |

### Reference: EntropySimilarityWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (ms_entropy) | 4.283e-4 ± 4.09e-3 (n=280820) | 6.102e-4 ± 4.89e-3 (n=338350) | 3.508e-4 ± 2.34e-3 (n=256184) | 4.293e-4 ± 2.11e-3 (n=96817) | 1.124e-3 ± 5.80e-3 (n=24006) | 4.053e-4 ± 1.28e-3 (n=3746) | 3.215e-4 ± 5.03e-4 (n=77) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosineHungarian (matchms) | 1.670e-3 ± 1.91e-2 (n=280820) | 7.768e-4 ± 8.20e-3 (n=338350) | 9.364e-4 ± 1.01e-2 (n=256184) | 2.249e-4 ± 1.95e-3 (n=96817) | 9.172e-6 ± 5.28e-5 (n=24006) | 5.498e-6 ± 2.17e-5 (n=3746) | 1.031e-16 ± 1.17e-17 (n=77) |
| ModifiedGreedyCosine (matchms) | 1.677e-3 ± 1.91e-2 (n=280820) | 8.205e-4 ± 8.24e-3 (n=338350) | 9.394e-4 ± 1.01e-2 (n=256184) | 3.188e-4 ± 2.13e-3 (n=96817) | 7.718e-5 ± 4.16e-4 (n=24006) | 6.747e-5 ± 2.23e-4 (n=3746) | 1.616e-9 ± 2.53e-9 (n=77) |

### Reference: ModifiedCosineMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.000e-16 ± 3.71e-19 (n=280820) | 1.000e-16 ± 4.49e-19 (n=338350) | 1.000e-16 ± 5.26e-19 (n=256184) | 1.000e-16 ± 8.99e-19 (n=96817) | 1.001e-16 ± 2.24e-18 (n=24006) | 1.001e-16 ± 1.78e-18 (n=3746) | 1.000e-16 (n=77) |

## Spectral Similarity vs Structural Similarity (Tanimoto) (Spectra used: 199026)

Y-axis: `Mean spectral similarity`

### atompair

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 3.673e-3 ± 2.97e-2 (n=214150) | 5.391e-3 ± 3.59e-2 (n=395049) | 6.081e-3 ± 3.81e-2 (n=288985) | 7.225e-3 ± 4.56e-2 (n=81329) | 2.153e-2 ± 1.09e-1 (n=13533) | 1.117e-1 ± 2.81e-1 (n=3575) | 2.031e-1 ± 3.75e-1 (n=1899) | 3.243e-1 ± 4.51e-1 (n=889) | 3.535e-1 ± 4.56e-1 (n=305) | 4.034e-1 ± 4.22e-1 (n=286) |
| CosineHungarianMerged (mass-spectrometry-traits) | 3.650e-3 ± 2.97e-2 (n=214150) | 5.376e-3 ± 3.61e-2 (n=395049) | 6.057e-3 ± 3.81e-2 (n=288985) | 7.214e-3 ± 4.57e-2 (n=81329) | 2.150e-2 ± 1.09e-1 (n=13533) | 1.118e-1 ± 2.80e-1 (n=3575) | 2.037e-1 ± 3.75e-1 (n=1899) | 3.256e-1 ± 4.50e-1 (n=889) | 3.560e-1 ± 4.55e-1 (n=305) | 4.013e-1 ± 4.24e-1 (n=286) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 5.534e-3 ± 2.85e-2 (n=214150) | 8.264e-3 ± 3.51e-2 (n=395049) | 9.543e-3 ± 3.81e-2 (n=288985) | 1.105e-2 ± 4.49e-2 (n=81329) | 2.524e-2 ± 9.64e-2 (n=13533) | 1.052e-1 ± 2.44e-1 (n=3575) | 1.909e-1 ± 3.36e-1 (n=1899) | 3.107e-1 ± 4.12e-1 (n=889) | 3.358e-1 ± 4.16e-1 (n=305) | 4.000e-1 ± 3.87e-1 (n=286) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 6.981e-3 ± 3.06e-2 (n=214150) | 1.008e-2 ± 3.72e-2 (n=395049) | 1.150e-2 ± 3.99e-2 (n=288985) | 1.302e-2 ± 4.57e-2 (n=81329) | 2.589e-2 ± 8.17e-2 (n=13533) | 8.778e-2 ± 1.84e-1 (n=3575) | 1.521e-1 ± 2.49e-1 (n=1899) | 2.385e-1 ± 2.99e-1 (n=889) | 2.633e-1 ± 3.09e-1 (n=305) | 3.858e-1 ± 3.50e-1 (n=286) |
| ModifiedCosine (mass-spectrometry-traits) | 5.859e-2 ± 1.73e-1 (n=214150) | 5.334e-2 ± 1.65e-1 (n=395049) | 5.395e-2 ± 1.66e-1 (n=288985) | 5.796e-2 ± 1.75e-1 (n=81329) | 8.141e-2 ± 2.12e-1 (n=13533) | 1.745e-1 ± 3.22e-1 (n=3575) | 2.695e-1 ± 4.00e-1 (n=1899) | 3.746e-1 ± 4.54e-1 (n=889) | 4.193e-1 ± 4.61e-1 (n=305) | 4.388e-1 ± 4.22e-1 (n=286) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.593e-2 ± 1.38e-1 (n=214150) | 5.197e-2 ± 1.31e-1 (n=395049) | 5.297e-2 ± 1.32e-1 (n=288985) | 5.644e-2 ± 1.40e-1 (n=81329) | 7.733e-2 ± 1.72e-1 (n=13533) | 1.605e-1 ± 2.72e-1 (n=3575) | 2.497e-1 ± 3.55e-1 (n=1899) | 3.539e-1 ± 4.12e-1 (n=889) | 3.944e-1 ± 4.20e-1 (n=305) | 4.306e-1 ± 3.83e-1 (n=286) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 5.534e-2 ± 1.11e-1 (n=214150) | 5.147e-2 ± 1.05e-1 (n=395049) | 5.231e-2 ± 1.06e-1 (n=288985) | 5.513e-2 ± 1.12e-1 (n=81329) | 7.307e-2 ± 1.40e-1 (n=13533) | 1.392e-1 ± 2.11e-1 (n=3575) | 2.055e-1 ± 2.73e-1 (n=1899) | 2.787e-1 ± 3.05e-1 (n=889) | 3.164e-1 ± 3.22e-1 (n=305) | 4.109e-1 ± 3.42e-1 (n=286) |

### ecfp

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 4.045e-3 ± 2.93e-2 (n=591518) | 6.661e-3 ± 4.05e-2 (n=372078) | 1.613e-2 ± 7.73e-2 (n=26152) | 2.809e-2 ± 1.09e-1 (n=4680) | 6.429e-2 ± 1.89e-1 (n=1705) | 2.285e-1 ± 3.85e-1 (n=1366) | 3.065e-1 ± 4.09e-1 (n=776) | 3.793e-1 ± 4.49e-1 (n=493) | 4.635e-1 ± 4.81e-1 (n=463) | 4.729e-1 ± 4.71e-1 (n=769) |
| CosineHungarianMerged (mass-spectrometry-traits) | 4.028e-3 ± 2.94e-2 (n=591518) | 6.641e-3 ± 4.06e-2 (n=372078) | 1.607e-2 ± 7.73e-2 (n=26152) | 2.807e-2 ± 1.09e-1 (n=4680) | 6.413e-2 ± 1.89e-1 (n=1705) | 2.290e-1 ± 3.85e-1 (n=1366) | 3.070e-1 ± 4.09e-1 (n=776) | 3.811e-1 ± 4.48e-1 (n=493) | 4.649e-1 ± 4.79e-1 (n=463) | 4.734e-1 ± 4.70e-1 (n=769) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.375e-3 ± 2.93e-2 (n=591518) | 1.033e-2 ± 4.00e-2 (n=372078) | 2.159e-2 ± 7.08e-2 (n=26152) | 3.524e-2 ± 9.84e-2 (n=4680) | 6.839e-2 ± 1.64e-1 (n=1705) | 2.120e-1 ± 3.40e-1 (n=1366) | 2.754e-1 ± 3.48e-1 (n=776) | 3.452e-1 ± 3.94e-1 (n=493) | 4.273e-1 ± 4.36e-1 (n=463) | 4.511e-1 ± 4.33e-1 (n=769) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.789e-3 ± 3.14e-2 (n=591518) | 1.258e-2 ± 4.22e-2 (n=372078) | 2.533e-2 ± 6.94e-2 (n=26152) | 4.004e-2 ± 9.42e-2 (n=4680) | 7.013e-2 ± 1.41e-1 (n=1705) | 1.688e-1 ± 2.52e-1 (n=1366) | 2.193e-1 ± 2.56e-1 (n=776) | 2.637e-1 ± 2.86e-1 (n=493) | 3.176e-1 ± 3.18e-1 (n=463) | 3.662e-1 ± 3.39e-1 (n=769) |
| ModifiedCosine (mass-spectrometry-traits) | 5.217e-2 ± 1.64e-1 (n=591518) | 5.810e-2 ± 1.71e-1 (n=372078) | 7.862e-2 ± 1.96e-1 (n=26152) | 1.026e-1 ± 2.25e-1 (n=4680) | 1.467e-1 ± 2.75e-1 (n=1705) | 2.844e-1 ± 4.01e-1 (n=1366) | 3.613e-1 ± 4.17e-1 (n=776) | 4.246e-1 ± 4.48e-1 (n=493) | 5.083e-1 ± 4.75e-1 (n=463) | 4.839e-1 ± 4.68e-1 (n=769) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 4.988e-2 ± 1.30e-1 (n=591518) | 5.742e-2 ± 1.37e-1 (n=372078) | 7.965e-2 ± 1.61e-1 (n=26152) | 1.039e-1 ± 1.87e-1 (n=4680) | 1.447e-1 ± 2.31e-1 (n=1705) | 2.623e-1 ± 3.50e-1 (n=1366) | 3.267e-1 ± 3.55e-1 (n=776) | 3.878e-1 ± 3.92e-1 (n=493) | 4.647e-1 ± 4.31e-1 (n=463) | 4.599e-1 ± 4.31e-1 (n=769) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 4.879e-2 ± 1.03e-1 (n=591518) | 5.725e-2 ± 1.10e-1 (n=372078) | 8.094e-2 ± 1.34e-1 (n=26152) | 1.054e-1 ± 1.61e-1 (n=4680) | 1.427e-1 ± 1.98e-1 (n=1705) | 2.157e-1 ± 2.65e-1 (n=1366) | 2.686e-1 ± 2.74e-1 (n=776) | 3.054e-1 ± 2.92e-1 (n=493) | 3.514e-1 ± 3.20e-1 (n=463) | 3.750e-1 ± 3.39e-1 (n=769) |

### fcfp

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 3.892e-3 ± 2.85e-2 (n=392325) | 5.336e-3 ± 3.55e-2 (n=490827) | 9.768e-3 ± 5.26e-2 (n=94350) | 1.866e-2 ± 8.20e-2 (n=13272) | 5.960e-2 ± 1.95e-1 (n=4200) | 1.240e-1 ± 2.89e-1 (n=1875) | 1.634e-1 ± 3.16e-1 (n=907) | 3.287e-1 ± 4.24e-1 (n=742) | 3.812e-1 ± 4.58e-1 (n=484) | 4.662e-1 ± 4.74e-1 (n=1018) |
| CosineHungarianMerged (mass-spectrometry-traits) | 3.872e-3 ± 2.86e-2 (n=392325) | 5.319e-3 ± 3.56e-2 (n=490827) | 9.736e-3 ± 5.26e-2 (n=94350) | 1.861e-2 ± 8.21e-2 (n=13272) | 5.981e-2 ± 1.96e-1 (n=4200) | 1.241e-1 ± 2.88e-1 (n=1875) | 1.638e-1 ± 3.15e-1 (n=907) | 3.295e-1 ± 4.23e-1 (n=742) | 3.819e-1 ± 4.57e-1 (n=484) | 4.669e-1 ± 4.73e-1 (n=1018) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.035e-3 ± 2.80e-2 (n=392325) | 8.417e-3 ± 3.51e-2 (n=490827) | 1.456e-2 ± 5.17e-2 (n=94350) | 2.506e-2 ± 7.63e-2 (n=13272) | 6.211e-2 ± 1.73e-1 (n=4200) | 1.223e-1 ± 2.56e-1 (n=1875) | 1.547e-1 ± 2.65e-1 (n=907) | 2.968e-1 ± 3.60e-1 (n=742) | 3.534e-1 ± 4.12e-1 (n=484) | 4.438e-1 ± 4.35e-1 (n=1018) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.404e-3 ± 3.02e-2 (n=392325) | 1.028e-2 ± 3.72e-2 (n=490827) | 1.740e-2 ± 5.30e-2 (n=94350) | 2.918e-2 ± 7.56e-2 (n=13272) | 5.875e-2 ± 1.38e-1 (n=4200) | 1.073e-1 ± 1.96e-1 (n=1875) | 1.347e-1 ± 2.06e-1 (n=907) | 2.332e-1 ± 2.63e-1 (n=742) | 2.710e-1 ± 2.96e-1 (n=484) | 3.554e-1 ± 3.37e-1 (n=1018) |
| ModifiedCosine (mass-spectrometry-traits) | 5.447e-2 ± 1.68e-1 (n=392325) | 5.316e-2 ± 1.64e-1 (n=490827) | 6.285e-2 ± 1.76e-1 (n=94350) | 8.656e-2 ± 2.04e-1 (n=13272) | 1.357e-1 ± 2.71e-1 (n=4200) | 2.120e-1 ± 3.39e-1 (n=1875) | 2.578e-1 ± 3.56e-1 (n=907) | 3.975e-1 ± 4.29e-1 (n=742) | 4.433e-1 ± 4.57e-1 (n=484) | 4.883e-1 ± 4.70e-1 (n=1018) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.142e-2 ± 1.33e-1 (n=392325) | 5.229e-2 ± 1.31e-1 (n=490827) | 6.336e-2 ± 1.42e-1 (n=94350) | 8.732e-2 ± 1.66e-1 (n=13272) | 1.308e-1 ± 2.28e-1 (n=4200) | 2.013e-1 ± 2.89e-1 (n=1875) | 2.437e-1 ± 2.97e-1 (n=907) | 3.573e-1 ± 3.63e-1 (n=742) | 4.118e-1 ± 4.10e-1 (n=484) | 4.625e-1 ± 4.32e-1 (n=1018) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 4.996e-2 ± 1.05e-1 (n=392325) | 5.197e-2 ± 1.05e-1 (n=490827) | 6.397e-2 ± 1.17e-1 (n=94350) | 8.862e-2 ± 1.40e-1 (n=13272) | 1.224e-1 ± 1.82e-1 (n=4200) | 1.812e-1 ± 2.27e-1 (n=1875) | 2.181e-1 ± 2.38e-1 (n=907) | 2.912e-1 ± 2.77e-1 (n=742) | 3.250e-1 ± 3.04e-1 (n=484) | 3.736e-1 ± 3.37e-1 (n=1018) |

### maccs

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 4.463e-3 ± 3.42e-2 (n=22378) | 4.656e-3 ± 3.31e-2 (n=144834) | 4.752e-3 ± 3.30e-2 (n=278164) | 5.179e-3 ± 3.47e-2 (n=271035) | 5.686e-3 ± 3.74e-2 (n=167595) | 7.195e-3 ± 4.46e-2 (n=75499) | 1.177e-2 ± 6.32e-2 (n=25206) | 4.804e-2 ± 1.80e-1 (n=9002) | 4.004e-2 ± 1.43e-1 (n=2955) | 3.019e-1 ± 4.31e-1 (n=3332) |
| CosineHungarianMerged (mass-spectrometry-traits) | 4.469e-3 ± 3.45e-2 (n=22378) | 4.623e-3 ± 3.30e-2 (n=144834) | 4.724e-3 ± 3.30e-2 (n=278164) | 5.167e-3 ± 3.48e-2 (n=271035) | 5.672e-3 ± 3.76e-2 (n=167595) | 7.173e-3 ± 4.46e-2 (n=75499) | 1.176e-2 ± 6.33e-2 (n=25206) | 4.814e-2 ± 1.80e-1 (n=9002) | 4.016e-2 ± 1.43e-1 (n=2955) | 3.025e-1 ± 4.30e-1 (n=3332) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.379e-3 ± 3.16e-2 (n=22378) | 7.040e-3 ± 3.20e-2 (n=144834) | 7.424e-3 ± 3.25e-2 (n=278164) | 8.109e-3 ± 3.43e-2 (n=271035) | 8.918e-3 ± 3.73e-2 (n=167595) | 1.096e-2 ± 4.41e-2 (n=75499) | 1.615e-2 ± 5.95e-2 (n=25206) | 5.023e-2 ± 1.62e-1 (n=9002) | 4.643e-2 ± 1.28e-1 (n=2955) | 2.808e-1 ± 3.86e-1 (n=3332) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.807e-3 ± 3.30e-2 (n=22378) | 8.716e-3 ± 3.42e-2 (n=144834) | 9.148e-3 ± 3.47e-2 (n=278164) | 9.869e-3 ± 3.64e-2 (n=271035) | 1.077e-2 ± 3.90e-2 (n=167595) | 1.305e-2 ± 4.55e-2 (n=75499) | 1.849e-2 ± 5.86e-2 (n=25206) | 4.647e-2 ± 1.27e-1 (n=9002) | 4.951e-2 ± 1.16e-1 (n=2955) | 2.222e-1 ± 2.92e-1 (n=3332) |
| ModifiedCosine (mass-spectrometry-traits) | 5.610e-2 ± 1.62e-1 (n=22378) | 5.426e-2 ± 1.65e-1 (n=144834) | 5.245e-2 ± 1.63e-1 (n=278164) | 5.261e-2 ± 1.64e-1 (n=271035) | 5.601e-2 ± 1.71e-1 (n=167595) | 6.368e-2 ± 1.83e-1 (n=75499) | 7.526e-2 ± 1.97e-1 (n=25206) | 1.226e-1 ± 2.64e-1 (n=9002) | 1.342e-1 ± 2.64e-1 (n=2955) | 3.590e-1 ± 4.37e-1 (n=3332) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.320e-2 ± 1.29e-1 (n=22378) | 5.258e-2 ± 1.31e-1 (n=144834) | 5.139e-2 ± 1.30e-1 (n=278164) | 5.134e-2 ± 1.30e-1 (n=271035) | 5.436e-2 ± 1.36e-1 (n=167595) | 6.113e-2 ± 1.46e-1 (n=75499) | 7.168e-2 ± 1.57e-1 (n=25206) | 1.138e-1 ± 2.20e-1 (n=9002) | 1.287e-1 ± 2.18e-1 (n=2955) | 3.315e-1 ± 3.87e-1 (n=3332) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 5.299e-2 ± 1.05e-1 (n=22378) | 5.244e-2 ± 1.05e-1 (n=144834) | 5.119e-2 ± 1.05e-1 (n=278164) | 5.083e-2 ± 1.04e-1 (n=271035) | 5.332e-2 ± 1.08e-1 (n=167595) | 5.931e-2 ± 1.16e-1 (n=75499) | 6.854e-2 ± 1.25e-1 (n=25206) | 1.023e-1 ± 1.71e-1 (n=9002) | 1.236e-1 ± 1.84e-1 (n=2955) | 2.695e-1 ± 2.97e-1 (n=3332) |

### map

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 5.984e-3 ± 3.78e-2 (n=494015) | 5.472e-3 ± 3.71e-2 (n=373416) | 4.177e-3 ± 3.56e-2 (n=95139) | 8.081e-3 ± 6.91e-2 (n=24356) | 4.425e-2 ± 1.92e-1 (n=8806) | 1.089e-1 ± 2.89e-1 (n=2624) | 2.982e-1 ± 4.44e-1 (n=879) | 4.210e-1 ± 4.83e-1 (n=404) | 3.928e-1 ± 4.81e-1 (n=107) | 4.181e-1 ± 4.18e-1 (n=254) |
| CosineHungarianMerged (mass-spectrometry-traits) | 5.952e-3 ± 3.78e-2 (n=494015) | 5.459e-3 ± 3.72e-2 (n=373416) | 4.190e-3 ± 3.58e-2 (n=95139) | 8.075e-3 ± 6.91e-2 (n=24356) | 4.435e-2 ± 1.92e-1 (n=8806) | 1.088e-1 ± 2.89e-1 (n=2624) | 3.000e-1 ± 4.44e-1 (n=879) | 4.230e-1 ± 4.81e-1 (n=404) | 3.935e-1 ± 4.80e-1 (n=107) | 4.156e-1 ± 4.20e-1 (n=254) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 9.184e-3 ± 3.72e-2 (n=494015) | 8.406e-3 ± 3.63e-2 (n=373416) | 6.305e-3 ± 3.52e-2 (n=95139) | 9.400e-3 ± 6.10e-2 (n=24356) | 4.135e-2 ± 1.70e-1 (n=8806) | 9.822e-2 ± 2.54e-1 (n=2624) | 2.824e-1 ± 4.07e-1 (n=879) | 3.948e-1 ± 4.43e-1 (n=404) | 3.701e-1 ± 4.43e-1 (n=107) | 4.154e-1 ± 3.82e-1 (n=254) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.139e-2 ± 3.99e-2 (n=494015) | 9.997e-3 ± 3.74e-2 (n=373416) | 7.316e-3 ± 3.54e-2 (n=95139) | 9.624e-3 ± 5.27e-2 (n=24356) | 3.272e-2 ± 1.26e-1 (n=8806) | 7.593e-2 ± 1.87e-1 (n=2624) | 2.138e-1 ± 2.98e-1 (n=879) | 2.922e-1 ± 3.20e-1 (n=404) | 2.845e-1 ± 3.34e-1 (n=107) | 4.061e-1 ± 3.50e-1 (n=254) |
| ModifiedCosine (mass-spectrometry-traits) | 6.246e-2 ± 1.75e-1 (n=494015) | 5.247e-2 ± 1.66e-1 (n=373416) | 3.958e-2 ± 1.48e-1 (n=95139) | 3.761e-2 ± 1.53e-1 (n=24356) | 7.191e-2 ± 2.30e-1 (n=8806) | 1.373e-1 ± 3.11e-1 (n=2624) | 3.240e-1 ± 4.49e-1 (n=879) | 4.504e-1 ± 4.81e-1 (n=404) | 4.560e-1 ± 4.81e-1 (n=107) | 4.344e-1 ± 4.15e-1 (n=254) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 6.155e-2 ± 1.41e-1 (n=494015) | 5.037e-2 ± 1.31e-1 (n=373416) | 3.717e-2 ± 1.17e-1 (n=95139) | 3.420e-2 ± 1.22e-1 (n=24356) | 6.380e-2 ± 1.96e-1 (n=8806) | 1.218e-1 ± 2.69e-1 (n=2624) | 3.042e-1 ± 4.10e-1 (n=879) | 4.189e-1 ± 4.41e-1 (n=404) | 4.268e-1 ± 4.45e-1 (n=107) | 4.286e-1 ± 3.76e-1 (n=254) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 6.177e-2 ± 1.14e-1 (n=494015) | 4.900e-2 ± 1.03e-1 (n=373416) | 3.532e-2 ± 9.29e-2 (n=95139) | 3.122e-2 ± 9.75e-2 (n=24356) | 5.233e-2 ± 1.49e-1 (n=8806) | 9.697e-2 ± 2.03e-1 (n=2624) | 2.349e-1 ± 3.05e-1 (n=879) | 3.134e-1 ± 3.23e-1 (n=404) | 3.389e-1 ± 3.45e-1 (n=107) | 4.163e-1 ± 3.43e-1 (n=254) |

### rdkit

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 4.689e-3 ± 3.36e-2 (n=145598) | 5.515e-3 ± 3.66e-2 (n=344476) | 5.778e-3 ± 3.74e-2 (n=279105) | 6.954e-3 ± 5.41e-2 (n=144007) | 5.426e-3 ± 3.89e-2 (n=58790) | 7.459e-3 ± 5.14e-2 (n=18361) | 2.185e-2 ± 1.10e-1 (n=4991) | 1.144e-1 ± 2.77e-1 (n=1892) | 2.231e-1 ± 3.72e-1 (n=1112) | 3.739e-1 ± 4.58e-1 (n=1668) |
| CosineHungarianMerged (mass-spectrometry-traits) | 4.668e-3 ± 3.36e-2 (n=145598) | 5.487e-3 ± 3.66e-2 (n=344476) | 5.759e-3 ± 3.75e-2 (n=279105) | 6.948e-3 ± 5.41e-2 (n=144007) | 5.414e-3 ± 3.89e-2 (n=58790) | 7.430e-3 ± 5.14e-2 (n=18361) | 2.187e-2 ± 1.09e-1 (n=4991) | 1.147e-1 ± 2.77e-1 (n=1892) | 2.231e-1 ± 3.71e-1 (n=1112) | 3.750e-1 ± 4.57e-1 (n=1668) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.940e-3 ± 3.23e-2 (n=145598) | 8.493e-3 ± 3.60e-2 (n=344476) | 9.006e-3 ± 3.71e-2 (n=279105) | 9.786e-3 ± 5.09e-2 (n=144007) | 8.308e-3 ± 3.86e-2 (n=58790) | 1.050e-2 ± 4.83e-2 (n=18361) | 2.389e-2 ± 9.41e-2 (n=4991) | 1.045e-1 ± 2.31e-1 (n=1892) | 2.067e-1 ± 3.21e-1 (n=1112) | 3.551e-1 ± 4.20e-1 (n=1668) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 8.741e-3 ± 3.49e-2 (n=145598) | 1.043e-2 ± 3.83e-2 (n=344476) | 1.083e-2 ± 3.88e-2 (n=279105) | 1.100e-2 ± 4.53e-2 (n=144007) | 9.738e-3 ± 3.93e-2 (n=58790) | 1.209e-2 ± 4.67e-2 (n=18361) | 2.436e-2 ± 8.17e-2 (n=4991) | 8.853e-2 ± 1.76e-1 (n=1892) | 1.668e-1 ± 2.40e-1 (n=1112) | 2.846e-1 ± 3.21e-1 (n=1668) |
| ModifiedCosine (mass-spectrometry-traits) | 6.016e-2 ± 1.70e-1 (n=145598) | 5.076e-2 ± 1.59e-1 (n=344476) | 5.506e-2 ± 1.69e-1 (n=279105) | 6.060e-2 ± 1.81e-1 (n=144007) | 5.898e-2 ± 1.79e-1 (n=58790) | 6.488e-2 ± 1.90e-1 (n=18361) | 8.142e-2 ± 2.14e-1 (n=4991) | 1.821e-1 ± 3.24e-1 (n=1892) | 2.908e-1 ± 3.95e-1 (n=1112) | 4.081e-1 ± 4.57e-1 (n=1668) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.892e-2 ± 1.36e-1 (n=145598) | 5.050e-2 ± 1.28e-1 (n=344476) | 5.331e-2 ± 1.34e-1 (n=279105) | 5.724e-2 ± 1.44e-1 (n=144007) | 5.547e-2 ± 1.41e-1 (n=58790) | 5.906e-2 ± 1.48e-1 (n=18361) | 7.441e-2 ± 1.71e-1 (n=4991) | 1.633e-1 ± 2.69e-1 (n=1892) | 2.638e-1 ± 3.37e-1 (n=1112) | 3.861e-1 ± 4.17e-1 (n=1668) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 6.054e-2 ± 1.13e-1 (n=145598) | 5.074e-2 ± 1.04e-1 (n=344476) | 5.199e-2 ± 1.06e-1 (n=279105) | 5.460e-2 ± 1.12e-1 (n=144007) | 5.237e-2 ± 1.10e-1 (n=58790) | 5.470e-2 ± 1.16e-1 (n=18361) | 6.792e-2 ± 1.37e-1 (n=4991) | 1.415e-1 ± 2.11e-1 (n=1892) | 2.190e-1 ± 2.59e-1 (n=1112) | 3.137e-1 ± 3.22e-1 (n=1668) |

## Correlation: Spectral Similarity vs Structural Similarity

| Fingerprint | Algorithm | Pearson r | Pearson p | Spearman rho | Spearman p | n_pairs |
| --- | --- | --- | --- | --- | --- | --- |
| atompair | CosineHungarian (mass-spectrometry-traits) | 0.1596 | 0.00e0 | 0.0865 | 0.00e0 | 1000000 |
| atompair | CosineHungarianMerged (mass-spectrometry-traits) | 0.1598 | 0.00e0 | 0.0874 | 0.00e0 | 1000000 |
| atompair | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1726 | 0.00e0 | 0.0812 | 0.00e0 | 1000000 |
| atompair | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.1594 | 0.00e0 | 0.0807 | 0.00e0 | 1000000 |
| atompair | ModifiedCosine (mass-spectrometry-traits) | 0.0427 | 0.00e0 | 0.0238 | 0.00e0 | 1000000 |
| atompair | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.0518 | 0.00e0 | 0.0262 | 0.00e0 | 1000000 |
| atompair | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.0504 | 0.00e0 | 0.0237 | 0.00e0 | 1000000 |
| ecfp | CosineHungarian (mass-spectrometry-traits) | 0.3193 | 0.00e0 | 0.1190 | 0.00e0 | 1000000 |
| ecfp | CosineHungarianMerged (mass-spectrometry-traits) | 0.3195 | 0.00e0 | 0.1194 | 0.00e0 | 1000000 |
| ecfp | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.3301 | 0.00e0 | 0.1072 | 0.00e0 | 1000000 |
| ecfp | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.3001 | 0.00e0 | 0.1079 | 0.00e0 | 1000000 |
| ecfp | ModifiedCosine (mass-spectrometry-traits) | 0.1066 | 0.00e0 | 0.0804 | 0.00e0 | 1000000 |
| ecfp | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.1305 | 0.00e0 | 0.0832 | 0.00e0 | 1000000 |
| ecfp | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.1420 | 0.00e0 | 0.0842 | 0.00e0 | 1000000 |
| fcfp | CosineHungarian (mass-spectrometry-traits) | 0.2687 | 0.00e0 | 0.1053 | 0.00e0 | 1000000 |
| fcfp | CosineHungarianMerged (mass-spectrometry-traits) | 0.2689 | 0.00e0 | 0.1062 | 0.00e0 | 1000000 |
| fcfp | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.2837 | 0.00e0 | 0.0955 | 0.00e0 | 1000000 |
| fcfp | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.2634 | 0.00e0 | 0.0962 | 0.00e0 | 1000000 |
| fcfp | ModifiedCosine (mass-spectrometry-traits) | 0.0874 | 0.00e0 | 0.0606 | 0.00e0 | 1000000 |
| fcfp | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.1106 | 0.00e0 | 0.0625 | 0.00e0 | 1000000 |
| fcfp | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.1234 | 0.00e0 | 0.0634 | 0.00e0 | 1000000 |
| maccs | CosineHungarian (mass-spectrometry-traits) | 0.1366 | 0.00e0 | 0.0544 | 0.00e0 | 1000000 |
| maccs | CosineHungarianMerged (mass-spectrometry-traits) | 0.1369 | 0.00e0 | 0.0553 | 0.00e0 | 1000000 |
| maccs | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1460 | 0.00e0 | 0.0551 | 0.00e0 | 1000000 |
| maccs | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.1347 | 0.00e0 | 0.0549 | 0.00e0 | 1000000 |
| maccs | ModifiedCosine (mass-spectrometry-traits) | 0.0580 | 0.00e0 | 0.0189 | 0.00e0 | 1000000 |
| maccs | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.0663 | 0.00e0 | 0.0293 | 0.00e0 | 1000000 |
| maccs | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.0665 | 0.00e0 | 0.0281 | 0.00e0 | 1000000 |
| map | CosineHungarian (mass-spectrometry-traits) | 0.1351 | 0.00e0 | -0.0309 | 0.00e0 | 1000000 |
| map | CosineHungarianMerged (mass-spectrometry-traits) | 0.1355 | 0.00e0 | -0.0297 | 0.00e0 | 1000000 |
| map | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1244 | 0.00e0 | -0.0234 | 0.00e0 | 1000000 |
| map | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.0906 | 0.00e0 | -0.0246 | 0.00e0 | 1000000 |
| map | ModifiedCosine (mass-spectrometry-traits) | -0.0092 | 0.00e0 | -0.1160 | 0.00e0 | 1000000 |
| map | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | -0.0198 | 0.00e0 | -0.1128 | 0.00e0 | 1000000 |
| map | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | -0.0458 | 0.00e0 | -0.1168 | 0.00e0 | 1000000 |
| rdkit | CosineHungarian (mass-spectrometry-traits) | 0.1281 | 0.00e0 | 0.0277 | 0.00e0 | 1000000 |
| rdkit | CosineHungarianMerged (mass-spectrometry-traits) | 0.1284 | 0.00e0 | 0.0286 | 0.00e0 | 1000000 |
| rdkit | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1293 | 0.00e0 | 0.0346 | 0.00e0 | 1000000 |
| rdkit | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.1096 | 0.00e0 | 0.0338 | 0.00e0 | 1000000 |
| rdkit | ModifiedCosine (mass-spectrometry-traits) | 0.0453 | 0.00e0 | -0.0166 | 0.00e0 | 1000000 |
| rdkit | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.0451 | 0.00e0 | -0.0008 | 4.36e-1 | 1000000 |
| rdkit | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.0324 | 0.00e0 | -0.0043 | 1.56e-5 | 1000000 |

