# Benchmark Tables

## Run Scope

- Total spectra in DB: `199032`
- Spectra used in results: `199026`

## Timing by Peak Count (Spectra used: 199026)

Y-axis: `Mean time (µs)`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (matchms) | 1.795e1 ± 3.29e0 (n=280820) | 1.910e1 ± 4.42e0 (n=338350) | 2.152e1 ± 5.71e0 (n=256184) | 2.512e1 ± 6.58e0 (n=96817) | 2.969e1 ± 9.25e0 (n=24006) | 3.555e1 ± 1.03e1 (n=3746) | 4.470e1 ± 7.84e0 (n=77) |
| CosineHungarian (mass-spectrometry-traits) | 1.120e0 ± 1.01e0 (n=280820) | 1.449e0 ± 1.08e0 (n=338350) | 2.119e0 ± 1.21e0 (n=256184) | 3.360e0 ± 1.47e0 (n=96817) | 6.158e0 ± 4.53e0 (n=24006) | 9.951e0 ± 7.42e0 (n=3746) | 1.505e1 ± 5.79e0 (n=77) |
| CosineHungarian (matchms) | 2.714e1 ± 1.29e1 (n=280820) | 3.152e1 ± 1.71e1 (n=338350) | 4.044e1 ± 2.16e1 (n=256184) | 5.271e1 ± 2.46e1 (n=96817) | 7.101e1 ± 5.92e1 (n=24006) | 8.890e1 ± 8.47e1 (n=3746) | 1.182e2 ± 6.02e1 (n=77) |

### Reference: CosineHungarianMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarianMerged (mass-spectrometry-traits) | 1.033e0 ± 9.23e-1 (n=280820) | 1.330e0 ± 9.95e-1 (n=338350) | 1.940e0 ± 1.12e0 (n=256184) | 3.044e0 ± 1.36e0 (n=96817) | 4.966e0 ± 1.86e0 (n=24006) | 8.304e0 ± 3.07e0 (n=3746) | 1.355e1 ± 3.81e0 (n=77) |
| LinearCosine (mass-spectrometry-traits) | 1.004e0 ± 9.04e-1 (n=280820) | 1.264e0 ± 9.59e-1 (n=338350) | 1.765e0 ± 1.06e0 (n=256184) | 2.690e0 ± 1.21e0 (n=96817) | 4.252e0 ± 1.40e0 (n=24006) | 6.963e0 ± 1.66e0 (n=3746) | 1.057e1 ± 1.10e0 (n=77) |

### Reference: EntropySimilarityUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 4.463e-1 ± 3.10e-1 (n=280820) | 5.414e-1 ± 3.35e-1 (n=338350) | 7.174e-1 ± 3.95e-1 (n=256184) | 1.011e0 ± 5.23e-1 (n=96817) | 1.261e0 ± 7.47e-1 (n=24006) | 1.438e0 ± 9.73e-1 (n=3746) | 1.844e0 ± 1.12e0 (n=77) |
| EntropySimilarityUnweighted (ms_entropy) | 9.906e0 ± 3.13e0 (n=280820) | 1.062e1 ± 3.36e0 (n=338350) | 1.214e1 ± 3.74e0 (n=256184) | 1.521e1 ± 4.27e0 (n=96817) | 2.093e1 ± 4.77e0 (n=24006) | 3.053e1 ± 5.69e0 (n=3746) | 4.446e1 ± 4.72e0 (n=77) |

### Reference: EntropySimilarityWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.917e-1 ± 4.44e-1 (n=280820) | 9.682e-1 ± 4.73e-1 (n=338350) | 1.275e0 ± 5.56e-1 (n=256184) | 1.688e0 ± 7.05e-1 (n=96817) | 1.953e0 ± 9.57e-1 (n=24006) | 2.179e0 ± 1.24e0 (n=3746) | 2.715e0 ± 1.40e0 (n=77) |
| EntropySimilarityWeighted (ms_entropy) | 1.060e1 ± 3.16e0 (n=280820) | 1.136e1 ± 3.36e0 (n=338350) | 1.295e1 ± 3.75e0 (n=256184) | 1.606e1 ± 4.25e0 (n=96817) | 2.171e1 ± 4.73e0 (n=24006) | 3.133e1 ± 5.59e0 (n=3746) | 4.565e1 ± 6.27e0 (n=77) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosine (mass-spectrometry-traits) | 1.244e0 ± 1.07e0 (n=280820) | 1.606e0 ± 1.15e0 (n=338350) | 2.318e0 ± 1.30e0 (n=256184) | 3.710e0 ± 1.62e0 (n=96817) | 6.864e0 ± 4.71e0 (n=24006) | 1.140e1 ± 8.04e0 (n=3746) | 1.774e1 ± 6.18e0 (n=77) |
| ModifiedCosineHungarian (matchms) | 3.779e1 ± 1.86e1 (n=280820) | 4.252e1 ± 2.00e1 (n=338350) | 5.064e1 ± 2.10e1 (n=256184) | 6.215e1 ± 2.35e1 (n=96817) | 8.194e1 ± 5.23e1 (n=24006) | 1.036e2 ± 6.88e1 (n=3746) | 1.465e2 ± 7.40e1 (n=77) |
| ModifiedGreedyCosine (matchms) | 2.710e1 ± 5.30e0 (n=280820) | 2.857e1 ± 5.80e0 (n=338350) | 3.104e1 ± 6.11e0 (n=256184) | 3.446e1 ± 6.29e0 (n=96817) | 3.952e1 ± 8.79e0 (n=24006) | 4.608e1 ± 1.02e1 (n=3746) | 5.600e1 ± 8.88e0 (n=77) |

### Reference: ModifiedCosineMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosineMerged (mass-spectrometry-traits) | 1.152e0 ± 9.80e-1 (n=280820) | 1.481e0 ± 1.06e0 (n=338350) | 2.125e0 ± 1.21e0 (n=256184) | 3.362e0 ± 1.50e0 (n=96817) | 5.589e0 ± 2.17e0 (n=24006) | 9.587e0 ± 4.26e0 (n=3746) | 1.615e1 ± 5.47e0 (n=77) |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.144e0 ± 9.82e-1 (n=280820) | 1.445e0 ± 1.06e0 (n=338350) | 2.039e0 ± 1.20e0 (n=256184) | 3.177e0 ± 1.46e0 (n=96817) | 5.150e0 ± 1.86e0 (n=24006) | 8.568e0 ± 2.60e0 (n=3746) | 1.358e1 ± 2.66e0 (n=77) |

### Reference: ModifiedLinearEntropyUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.444e-1 ± 3.53e-1 (n=280820) | 6.590e-1 ± 3.90e-1 (n=338350) | 8.763e-1 ± 4.81e-1 (n=256184) | 1.243e0 ± 6.87e-1 (n=96817) | 1.552e0 ± 1.02e0 (n=24006) | 1.779e0 ± 1.37e0 (n=3746) | 2.284e0 ± 1.50e0 (n=77) |

### Reference: ModifiedLinearEntropyWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 8.898e-1 ± 4.83e-1 (n=280820) | 1.087e0 ± 5.24e-1 (n=338350) | 1.435e0 ± 6.36e-1 (n=256184) | 1.918e0 ± 8.55e-1 (n=96817) | 2.239e0 ± 1.22e0 (n=24006) | 2.512e0 ± 1.62e0 (n=3746) | 3.164e0 ± 1.77e0 (n=77) |

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

### Spectral Similarity vs Structural Similarity (Tanimoto)

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (matchms) | 4.045e-3 ± 2.93e-2 (n=591518) | 6.662e-3 ± 4.05e-2 (n=372078) | 1.613e-2 ± 7.73e-2 (n=26152) | 2.809e-2 ± 1.09e-1 (n=4680) | 6.429e-2 ± 1.89e-1 (n=1705) | 2.285e-1 ± 3.85e-1 (n=1366) | 3.065e-1 ± 4.09e-1 (n=776) | 3.792e-1 ± 4.49e-1 (n=493) | 4.635e-1 ± 4.81e-1 (n=463) | 4.729e-1 ± 4.71e-1 (n=769) |
| CosineHungarian (mass-spectrometry-traits) | 4.045e-3 ± 2.93e-2 (n=591518) | 6.661e-3 ± 4.05e-2 (n=372078) | 1.613e-2 ± 7.73e-2 (n=26152) | 2.809e-2 ± 1.09e-1 (n=4680) | 6.429e-2 ± 1.89e-1 (n=1705) | 2.285e-1 ± 3.85e-1 (n=1366) | 3.065e-1 ± 4.09e-1 (n=776) | 3.793e-1 ± 4.49e-1 (n=493) | 4.635e-1 ± 4.81e-1 (n=463) | 4.729e-1 ± 4.71e-1 (n=769) |
| CosineHungarian (matchms) | 4.045e-3 ± 2.93e-2 (n=591518) | 6.663e-3 ± 4.05e-2 (n=372078) | 1.613e-2 ± 7.73e-2 (n=26152) | 2.809e-2 ± 1.09e-1 (n=4680) | 6.429e-2 ± 1.89e-1 (n=1705) | 2.285e-1 ± 3.85e-1 (n=1366) | 3.065e-1 ± 4.09e-1 (n=776) | 3.793e-1 ± 4.49e-1 (n=493) | 4.635e-1 ± 4.81e-1 (n=463) | 4.729e-1 ± 4.71e-1 (n=769) |
| CosineHungarianMerged (mass-spectrometry-traits) | 4.028e-3 ± 2.94e-2 (n=591518) | 6.641e-3 ± 4.06e-2 (n=372078) | 1.607e-2 ± 7.73e-2 (n=26152) | 2.807e-2 ± 1.09e-1 (n=4680) | 6.413e-2 ± 1.89e-1 (n=1705) | 2.290e-1 ± 3.85e-1 (n=1366) | 3.070e-1 ± 4.09e-1 (n=776) | 3.811e-1 ± 4.48e-1 (n=493) | 4.649e-1 ± 4.79e-1 (n=463) | 4.734e-1 ± 4.70e-1 (n=769) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.375e-3 ± 2.93e-2 (n=591518) | 1.033e-2 ± 4.00e-2 (n=372078) | 2.159e-2 ± 7.08e-2 (n=26152) | 3.524e-2 ± 9.84e-2 (n=4680) | 6.839e-2 ± 1.64e-1 (n=1705) | 2.120e-1 ± 3.40e-1 (n=1366) | 2.754e-1 ± 3.48e-1 (n=776) | 3.452e-1 ± 3.94e-1 (n=493) | 4.273e-1 ± 4.36e-1 (n=463) | 4.511e-1 ± 4.33e-1 (n=769) |
| EntropySimilarityUnweighted (ms_entropy) | 6.375e-3 ± 2.93e-2 (n=591518) | 1.033e-2 ± 4.00e-2 (n=372078) | 2.158e-2 ± 7.08e-2 (n=26152) | 3.523e-2 ± 9.84e-2 (n=4680) | 6.840e-2 ± 1.64e-1 (n=1705) | 2.120e-1 ± 3.40e-1 (n=1366) | 2.753e-1 ± 3.47e-1 (n=776) | 3.453e-1 ± 3.94e-1 (n=493) | 4.273e-1 ± 4.36e-1 (n=463) | 4.511e-1 ± 4.33e-1 (n=769) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.789e-3 ± 3.14e-2 (n=591518) | 1.258e-2 ± 4.22e-2 (n=372078) | 2.533e-2 ± 6.94e-2 (n=26152) | 4.004e-2 ± 9.42e-2 (n=4680) | 7.013e-2 ± 1.41e-1 (n=1705) | 1.688e-1 ± 2.52e-1 (n=1366) | 2.193e-1 ± 2.56e-1 (n=776) | 2.637e-1 ± 2.86e-1 (n=493) | 3.176e-1 ± 3.18e-1 (n=463) | 3.662e-1 ± 3.39e-1 (n=769) |
| EntropySimilarityWeighted (ms_entropy) | 7.788e-3 ± 3.14e-2 (n=591518) | 1.258e-2 ± 4.22e-2 (n=372078) | 2.531e-2 ± 6.94e-2 (n=26152) | 4.003e-2 ± 9.42e-2 (n=4680) | 7.013e-2 ± 1.41e-1 (n=1705) | 1.688e-1 ± 2.52e-1 (n=1366) | 2.192e-1 ± 2.56e-1 (n=776) | 2.638e-1 ± 2.86e-1 (n=493) | 3.176e-1 ± 3.18e-1 (n=463) | 3.662e-1 ± 3.39e-1 (n=769) |
| LinearCosine (mass-spectrometry-traits) | 4.028e-3 ± 2.94e-2 (n=591518) | 6.641e-3 ± 4.06e-2 (n=372078) | 1.607e-2 ± 7.73e-2 (n=26152) | 2.807e-2 ± 1.09e-1 (n=4680) | 6.413e-2 ± 1.89e-1 (n=1705) | 2.290e-1 ± 3.85e-1 (n=1366) | 3.070e-1 ± 4.09e-1 (n=776) | 3.811e-1 ± 4.48e-1 (n=493) | 4.649e-1 ± 4.79e-1 (n=463) | 4.734e-1 ± 4.70e-1 (n=769) |
| ModifiedCosine (mass-spectrometry-traits) | 5.217e-2 ± 1.64e-1 (n=591518) | 5.810e-2 ± 1.71e-1 (n=372078) | 7.862e-2 ± 1.96e-1 (n=26152) | 1.026e-1 ± 2.25e-1 (n=4680) | 1.467e-1 ± 2.75e-1 (n=1705) | 2.844e-1 ± 4.01e-1 (n=1366) | 3.613e-1 ± 4.17e-1 (n=776) | 4.246e-1 ± 4.48e-1 (n=493) | 5.083e-1 ± 4.75e-1 (n=463) | 4.839e-1 ± 4.68e-1 (n=769) |
| ModifiedCosineHungarian (matchms) | 5.217e-2 ± 1.64e-1 (n=591518) | 5.811e-2 ± 1.71e-1 (n=372078) | 7.862e-2 ± 1.96e-1 (n=26152) | 1.026e-1 ± 2.25e-1 (n=4680) | 1.467e-1 ± 2.75e-1 (n=1705) | 2.844e-1 ± 4.01e-1 (n=1366) | 3.613e-1 ± 4.17e-1 (n=776) | 4.246e-1 ± 4.48e-1 (n=493) | 5.083e-1 ± 4.75e-1 (n=463) | 4.839e-1 ± 4.68e-1 (n=769) |
| ModifiedCosineMerged (mass-spectrometry-traits) | 5.202e-2 ± 1.64e-1 (n=591518) | 5.804e-2 ± 1.72e-1 (n=372078) | 7.865e-2 ± 1.97e-1 (n=26152) | 1.024e-1 ± 2.25e-1 (n=4680) | 1.466e-1 ± 2.75e-1 (n=1705) | 2.845e-1 ± 4.01e-1 (n=1366) | 3.618e-1 ± 4.16e-1 (n=776) | 4.265e-1 ± 4.47e-1 (n=493) | 5.096e-1 ± 4.72e-1 (n=463) | 4.842e-1 ± 4.68e-1 (n=769) |
| ModifiedGreedyCosine (matchms) | 5.217e-2 ± 1.64e-1 (n=591518) | 5.810e-2 ± 1.71e-1 (n=372078) | 7.861e-2 ± 1.96e-1 (n=26152) | 1.025e-1 ± 2.25e-1 (n=4680) | 1.467e-1 ± 2.75e-1 (n=1705) | 2.844e-1 ± 4.01e-1 (n=1366) | 3.613e-1 ± 4.17e-1 (n=776) | 4.245e-1 ± 4.48e-1 (n=493) | 5.083e-1 ± 4.74e-1 (n=463) | 4.839e-1 ± 4.68e-1 (n=769) |
| ModifiedLinearCosine (mass-spectrometry-traits) | 5.202e-2 ± 1.64e-1 (n=591518) | 5.804e-2 ± 1.72e-1 (n=372078) | 7.865e-2 ± 1.97e-1 (n=26152) | 1.024e-1 ± 2.25e-1 (n=4680) | 1.466e-1 ± 2.75e-1 (n=1705) | 2.845e-1 ± 4.01e-1 (n=1366) | 3.618e-1 ± 4.16e-1 (n=776) | 4.265e-1 ± 4.47e-1 (n=493) | 5.096e-1 ± 4.72e-1 (n=463) | 4.842e-1 ± 4.68e-1 (n=769) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 4.988e-2 ± 1.30e-1 (n=591518) | 5.742e-2 ± 1.37e-1 (n=372078) | 7.965e-2 ± 1.61e-1 (n=26152) | 1.039e-1 ± 1.87e-1 (n=4680) | 1.447e-1 ± 2.31e-1 (n=1705) | 2.623e-1 ± 3.50e-1 (n=1366) | 3.267e-1 ± 3.55e-1 (n=776) | 3.878e-1 ± 3.92e-1 (n=493) | 4.647e-1 ± 4.31e-1 (n=463) | 4.599e-1 ± 4.31e-1 (n=769) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 4.879e-2 ± 1.03e-1 (n=591518) | 5.725e-2 ± 1.10e-1 (n=372078) | 8.094e-2 ± 1.34e-1 (n=26152) | 1.054e-1 ± 1.61e-1 (n=4680) | 1.427e-1 ± 1.98e-1 (n=1705) | 2.157e-1 ± 2.65e-1 (n=1366) | 2.686e-1 ± 2.74e-1 (n=776) | 3.054e-1 ± 2.92e-1 (n=493) | 3.514e-1 ± 3.20e-1 (n=463) | 3.750e-1 ± 3.39e-1 (n=769) |

## Correlation: Spectral Similarity vs Structural Similarity

| Algorithm | Pearson r | Pearson p | Spearman rho | Spearman p | n_pairs |
| --- | --- | --- | --- | --- | --- |
| CosineGreedy (matchms) | 0.3193 | 0.00e0 | 0.1190 | 0.00e0 | 1000000 |
| CosineHungarian (mass-spectrometry-traits) | 0.3193 | 0.00e0 | 0.1190 | 0.00e0 | 1000000 |
| CosineHungarian (matchms) | 0.3193 | 0.00e0 | 0.1190 | 0.00e0 | 1000000 |
| CosineHungarianMerged (mass-spectrometry-traits) | 0.3195 | 0.00e0 | 0.1194 | 0.00e0 | 1000000 |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.3301 | 0.00e0 | 0.1072 | 0.00e0 | 1000000 |
| EntropySimilarityUnweighted (ms_entropy) | 0.3301 | 0.00e0 | 0.1072 | 0.00e0 | 1000000 |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 0.3001 | 0.00e0 | 0.1079 | 0.00e0 | 1000000 |
| EntropySimilarityWeighted (ms_entropy) | 0.3001 | 0.00e0 | 0.1079 | 0.00e0 | 1000000 |
| LinearCosine (mass-spectrometry-traits) | 0.3195 | 0.00e0 | 0.1194 | 0.00e0 | 1000000 |
| ModifiedCosine (mass-spectrometry-traits) | 0.1066 | 0.00e0 | 0.0804 | 0.00e0 | 1000000 |
| ModifiedCosineHungarian (matchms) | 0.1066 | 0.00e0 | 0.0804 | 0.00e0 | 1000000 |
| ModifiedCosineMerged (mass-spectrometry-traits) | 0.1069 | 0.00e0 | 0.0800 | 0.00e0 | 1000000 |
| ModifiedGreedyCosine (matchms) | 0.1066 | 0.00e0 | 0.0804 | 0.00e0 | 1000000 |
| ModifiedLinearCosine (mass-spectrometry-traits) | 0.1069 | 0.00e0 | 0.0800 | 0.00e0 | 1000000 |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.1305 | 0.00e0 | 0.0832 | 0.00e0 | 1000000 |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.1420 | 0.00e0 | 0.0842 | 0.00e0 | 1000000 |

