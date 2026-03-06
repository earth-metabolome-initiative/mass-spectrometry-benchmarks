# Benchmark Tables

## Run Scope

- Requested max spectra: `20`
- Total spectra in DB: `20`
- Spectra used in results: `20`

## Timing by Peak Count (Spectra used: 20)

Y-axis: `Mean time (µs)`

### Reference: CosineHungarian (matchms)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 1.772e0 ± 2.57e-1 (n=22) | 2.415e0 ± 4.70e-1 (n=64) | 3.610e0 ± 8.58e-1 (n=14) |
| CosineGreedy (matchms) | 2.056e1 ± 4.85e0 (n=22) | 2.605e1 ± 9.19e0 (n=64) | 3.412e1 ± 9.32e0 (n=14) |
| CosineHungarian (mass-spectrometry-traits) | 1.826e0 ± 3.01e-1 (n=22) | 3.355e0 ± 2.63e0 (n=64) | 8.609e0 ± 1.07e1 (n=14) |
| CosineHungarian (matchms) | 2.888e1 ± 1.84e1 (n=22) | 4.925e1 ± 3.81e1 (n=64) | 9.561e1 ± 7.34e1 (n=14) |

### Reference: CosineHungarianMerged (mass-spectrometry-traits)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| CosineHungarianMerged (mass-spectrometry-traits) | 1.729e0 ± 2.79e-1 (n=22) | 3.246e0 ± 2.79e0 (n=64) | 7.819e0 ± 9.69e0 (n=14) |
| LinearCosine (mass-spectrometry-traits) | 1.767e0 ± 2.74e-1 (n=22) | 2.159e0 ± 3.70e-1 (n=64) | 2.952e0 ± 2.28e-1 (n=14) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 5.670e-1 ± 2.50e-1 (n=22) | 5.035e-1 ± 2.52e-1 (n=64) | 7.714e-1 ± 1.98e-1 (n=14) |
| EntropySimilarityUnweighted (ms_entropy) | 1.176e1 ± 8.47e-1 (n=22) | 1.356e1 ± 1.25e0 (n=64) | 1.634e1 ± 7.13e-1 (n=14) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.054e0 ± 4.98e-1 (n=22) | 9.115e-1 ± 4.50e-1 (n=64) | 1.392e0 ± 2.84e-1 (n=14) |
| EntropySimilarityWeighted (ms_entropy) | 1.235e1 ± 1.02e0 (n=22) | 1.387e1 ± 1.40e0 (n=64) | 1.654e1 ± 8.95e-1 (n=14) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| ModifiedCosine (mass-spectrometry-traits) | 2.087e0 ± 2.83e-1 (n=22) | 4.059e0 ± 3.30e0 (n=64) | 1.089e1 ± 1.25e1 (n=14) |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 2.085e0 ± 2.94e-1 (n=22) | 2.869e0 ± 5.17e-1 (n=64) | 4.216e0 ± 7.63e-1 (n=14) |
| ModifiedGreedyCosine (matchms) | 3.094e1 ± 6.47e0 (n=22) | 4.018e1 ± 7.87e0 (n=64) | 4.605e1 ± 1.31e1 (n=14) |

### Reference: ModifiedCosineMerged (mass-spectrometry-traits)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| ModifiedCosineMerged (mass-spectrometry-traits) | 1.966e0 ± 2.63e-1 (n=22) | 3.685e0 ± 2.83e0 (n=64) | 9.380e0 ± 1.08e1 (n=14) |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.934e0 ± 2.81e-1 (n=22) | 2.532e0 ± 3.98e-1 (n=64) | 3.597e0 ± 5.74e-1 (n=14) |

## RMSE vs Reference by Peak Count (Spectra used: 20)

Y-axis: `RMSE`

### Reference: CosineHungarian (matchms)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 1.000e-16 ± 4.01e-25 (n=22) | 1.194e-16 ± 4.81e-17 (n=64) | 1.139e-16 ± 2.66e-17 (n=14) |
| CosineGreedy (matchms) | 1.000e-16 ± 4.01e-25 (n=22) | 1.107e-16 ± 3.02e-17 (n=64) | 1.008e-16 ± 1.93e-18 (n=14) |
| CosineHungarian (mass-spectrometry-traits) | 1.005e-16 ± 1.48e-18 (n=22) | 1.036e-16 ± 1.25e-17 (n=64) | 1.000e-16 ± 6.24e-25 (n=14) |

### Reference: CosineHungarianMerged (mass-spectrometry-traits)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| LinearCosine (mass-spectrometry-traits) | 1.000e-16 ± 4.01e-25 (n=22) | 1.000e-16 (n=64) | 1.000e-16 ± 6.24e-25 (n=14) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 4.927e-9 ± 5.06e-9 (n=22) | 2.013e-8 ± 2.44e-8 (n=64) | 2.939e-8 ± 2.51e-8 (n=14) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.445e-8 ± 1.50e-8 (n=22) | 1.385e-8 ± 1.48e-8 (n=64) | 1.757e-8 ± 1.65e-8 (n=14) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 1.005e-16 ± 1.48e-18 (n=22) | 1.167e-16 ± 4.67e-17 (n=64) | 1.132e-16 ± 2.63e-17 (n=14) |
| ModifiedGreedyCosine (matchms) | 1.227e-16 ± 3.83e-17 (n=22) | 1.138e-16 ± 3.24e-17 (n=64) | 1.000e-16 ± 6.24e-25 (n=14) |

### Reference: ModifiedCosineMerged (mass-spectrometry-traits)

| Series | 17–32 | 33–64 | 65–128 |
| --- | --- | --- | --- |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.011e-16 ± 2.19e-18 (n=22) | 1.435e-16 ± 1.04e-16 (n=64) | 2.296e-5 ± 2.50e-5 (n=14) |

