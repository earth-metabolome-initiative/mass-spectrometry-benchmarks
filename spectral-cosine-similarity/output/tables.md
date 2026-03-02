# Benchmark Tables

## Timing by Peak Count (Spectra used: 100)

Y-axis: `Mean time (µs)`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 2.334e0 ± 1.97e0 (n=400) | 2.719e0 ± 1.91e0 (n=788) | 3.150e0 ± 2.15e0 (n=2992) | 4.166e0 ± 2.19e0 (n=10920) | 6.548e0 ± 3.02e0 (n=4736) | 1.367e1 ± 5.01e0 (n=304) | 2.625e1 ± 1.29e1 (n=60) |
| CosineGreedy (matchms) | 2.219e1 ± 5.85e0 (n=400) | 2.487e1 ± 5.76e0 (n=788) | 2.603e1 ± 6.14e0 (n=2992) | 2.961e1 ± 8.06e0 (n=10920) | 3.753e1 ± 1.32e1 (n=4736) | 6.458e1 ± 4.06e1 (n=304) | 1.156e2 ± 7.59e1 (n=60) |
| CosineHungarian (mass-spectrometry-traits) | 2.387e0 ± 1.74e0 (n=400) | 2.874e0 ± 1.82e0 (n=788) | 3.542e0 ± 2.51e0 (n=2992) | 5.057e0 ± 3.26e0 (n=10920) | 9.935e0 ± 8.55e0 (n=4736) | 3.969e1 ± 4.72e1 (n=304) | 1.575e2 ± 1.77e2 (n=60) |
| CosineHungarian (matchms) | 3.788e1 ± 2.05e1 (n=400) | 4.900e1 ± 2.09e1 (n=788) | 5.337e1 ± 2.33e1 (n=2992) | 6.856e1 ± 4.07e1 (n=10920) | 1.154e2 ± 1.16e2 (n=4736) | 4.491e2 ± 6.51e2 (n=304) | 1.732e3 ± 2.26e3 (n=60) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 5.145e-1 ± 3.54e-1 (n=400) | 5.778e-1 ± 3.83e-1 (n=788) | 6.534e-1 ± 4.11e-1 (n=2992) | 8.818e-1 ± 4.55e-1 (n=10920) | 1.463e0 ± 6.91e-1 (n=4736) | 3.368e0 ± 1.36e0 (n=304) | 6.686e0 ± 2.57e0 (n=60) |
| EntropySimilarityUnweighted (ms_entropy) | 1.115e1 ± 5.99e-1 (n=400) | 1.104e1 ± 4.82e-1 (n=788) | 1.121e1 ± 8.71e-1 (n=2992) | 1.138e1 ± 8.51e-1 (n=10920) | 1.186e1 ± 8.40e-1 (n=4736) | 1.299e1 ± 1.00e0 (n=304) | 1.557e1 ± 2.41e0 (n=60) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.655e0 ± 8.56e-1 (n=400) | 1.971e0 ± 9.55e-1 (n=788) | 2.238e0 ± 1.06e0 (n=2992) | 2.919e0 ± 1.07e0 (n=10920) | 3.959e0 ± 1.38e0 (n=4736) | 6.185e0 ± 1.98e0 (n=304) | 1.049e1 ± 3.46e0 (n=60) |
| EntropySimilarityWeighted (ms_entropy) | 1.271e1 ± 1.61e0 (n=400) | 1.263e1 ± 1.18e0 (n=788) | 1.273e1 ± 1.03e0 (n=2992) | 1.334e1 ± 1.28e0 (n=10920) | 1.433e1 ± 1.47e0 (n=4736) | 1.630e1 ± 1.58e0 (n=304) | 1.873e1 ± 2.58e0 (n=60) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosine (mass-spectrometry-traits) | 2.751e0 ± 1.89e0 (n=400) | 3.285e0 ± 2.14e0 (n=788) | 4.256e0 ± 3.16e0 (n=2992) | 6.803e0 ± 5.37e0 (n=10920) | 1.475e1 ± 1.51e1 (n=4736) | 7.038e1 ± 7.53e1 (n=304) | 2.011e2 ± 1.86e2 (n=60) |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 2.620e0 ± 1.91e0 (n=400) | 3.047e0 ± 2.02e0 (n=788) | 3.570e0 ± 2.43e0 (n=2992) | 4.878e0 ± 2.60e0 (n=10920) | 7.888e0 ± 3.84e0 (n=4736) | 1.760e1 ± 7.35e0 (n=304) | 3.001e1 ± 1.31e1 (n=60) |
| ModifiedGreedyCosine (matchms) | 3.217e1 ± 5.71e0 (n=400) | 3.372e1 ± 5.66e0 (n=788) | 3.621e1 ± 6.49e0 (n=2992) | 4.120e1 ± 1.04e1 (n=10920) | 5.201e1 ± 2.13e1 (n=4736) | 1.035e2 ± 7.65e1 (n=304) | 1.872e2 ± 1.46e2 (n=60) |

## RMSE vs Reference by Peak Count (Spectra used: 100)

Y-axis: `RMSE`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 1.010e-16 ± 6.97e-18 (n=400) | 6.869e-5 ± 1.85e-4 (n=788) | 9.432e-5 ± 2.85e-4 (n=2992) | 4.471e-5 ± 1.39e-4 (n=10920) | 8.524e-5 ± 2.57e-4 (n=4736) | 5.758e-4 ± 1.23e-3 (n=304) | 1.148e-4 ± 1.59e-4 (n=60) |
| CosineGreedy (matchms) | 1.030e-16 ± 1.71e-17 (n=400) | 6.869e-5 ± 1.85e-4 (n=788) | 9.432e-5 ± 2.85e-4 (n=2992) | 4.471e-5 ± 1.39e-4 (n=10920) | 8.524e-5 ± 2.57e-4 (n=4736) | 5.758e-4 ± 1.23e-3 (n=304) | 1.148e-4 ± 1.59e-4 (n=60) |
| CosineHungarian (mass-spectrometry-traits) | 1.010e-16 ± 6.97e-18 (n=400) | 1.007e-16 ± 5.27e-18 (n=788) | 1.020e-16 ± 1.21e-17 (n=2992) | 1.027e-16 ± 1.53e-17 (n=10920) | 1.054e-16 ± 2.93e-17 (n=4736) | 1.106e-16 ± 2.50e-17 (n=304) | 1.755e-16 ± 9.40e-17 (n=60) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 1.027e-8 ± 9.68e-9 (n=400) | 1.159e-8 ± 1.48e-8 (n=788) | 1.385e-8 ± 1.78e-8 (n=2992) | 1.519e-8 ± 2.69e-8 (n=10920) | 2.395e-8 ± 4.01e-8 (n=4736) | 4.594e-8 ± 5.33e-8 (n=304) | 3.219e-8 ± 3.09e-8 (n=60) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.599e-8 ± 2.05e-8 (n=400) | 1.677e-8 ± 1.90e-8 (n=788) | 1.493e-8 ± 1.51e-8 (n=2992) | 2.194e-8 ± 3.30e-8 (n=10920) | 2.880e-8 ± 4.27e-8 (n=4736) | 4.873e-8 ± 5.99e-8 (n=304) | 3.219e-8 ± 3.09e-8 (n=60) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 1.552e-5 ± 3.38e-5 (n=400) | 1.731e-4 ± 4.31e-4 (n=788) | 8.246e-4 ± 2.55e-3 (n=2992) | 2.865e-4 ± 9.82e-4 (n=10920) | 2.153e-3 ± 8.02e-3 (n=4736) | 1.037e-3 ± 1.88e-3 (n=304) | 1.078e-3 ± 1.56e-3 (n=60) |
| ModifiedGreedyCosine (matchms) | 1.552e-5 ± 3.38e-5 (n=400) | 1.731e-4 ± 4.31e-4 (n=788) | 8.246e-4 ± 2.55e-3 (n=2992) | 2.865e-4 ± 9.82e-4 (n=10920) | 2.153e-3 ± 8.02e-3 (n=4736) | 1.037e-3 ± 1.88e-3 (n=304) | 1.078e-3 ± 1.56e-3 (n=60) |

