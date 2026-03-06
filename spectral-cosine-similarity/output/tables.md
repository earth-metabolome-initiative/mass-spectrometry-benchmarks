# Benchmark Tables

## Run Scope

- Requested max spectra: `100`
- Total spectra in DB: `100`
- Spectra used in results: `66`

## Timing by Peak Count (Spectra used: 66)

Y-axis: `Mean time (µs)`

### Reference: CosineHungarian (matchms)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 9.122e0 ± 1.78e0 (n=2) | 2.065e1 ± 1.01e1 (n=6) | 2.189e1 ± 8.35e0 (n=32) | 2.933e1 ± 7.54e0 (n=8) | 6.829e1 ± 1.56e1 (n=2) |
| CosineGreedy (matchms) | 1.901e1 ± 2.55e-1 (n=2) | 2.783e1 ± 7.18e0 (n=6) | 2.394e1 ± 7.02e0 (n=32) | 2.434e1 ± 6.90e0 (n=8) | 2.122e1 ± 1.41e-1 (n=2) |
| CosineHungarian (mass-spectrometry-traits) | 9.122e0 ± 1.69e0 (n=2) | 2.230e1 ± 1.14e1 (n=6) | 2.503e1 ± 1.65e1 (n=32) | 3.011e1 ± 7.91e0 (n=8) | 6.813e1 ± 1.59e1 (n=2) |
| CosineHungarian (matchms) | 1.787e1 ± 1.56e-1 (n=2) | 4.524e1 ± 2.11e1 (n=6) | 3.513e1 ± 2.43e1 (n=32) | 3.439e1 ± 2.15e1 (n=8) | 1.972e1 ± 2.76e-1 (n=2) |
| LinearCosine (mass-spectrometry-traits) | 1.144e1 ± 2.41e0 (n=2) | 2.706e1 ± 1.37e1 (n=6) | 2.835e1 ± 1.46e1 (n=32) | 3.645e1 ± 1.59e1 (n=8) | 9.173e1 ± 7.01e0 (n=2) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 2.415e0 ± 4.82e-1 (n=2) | 6.183e0 ± 4.40e0 (n=6) | 5.395e0 ± 4.05e0 (n=32) | 6.278e0 ± 3.63e0 (n=8) | 1.034e1 ± 5.53e0 (n=2) |
| EntropySimilarityUnweighted (ms_entropy) | 1.040e1 ± 1.06e0 (n=2) | 1.472e1 ± 5.82e0 (n=6) | 1.557e1 ± 4.91e0 (n=32) | 1.780e1 ± 4.63e0 (n=8) | 2.930e1 ± 6.20e0 (n=2) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 3.672e0 ± 7.16e-1 (n=2) | 8.755e0 ± 5.85e0 (n=6) | 7.399e0 ± 5.02e0 (n=32) | 8.660e0 ± 4.54e0 (n=8) | 1.368e1 ± 7.21e0 (n=2) |
| EntropySimilarityWeighted (ms_entropy) | 1.082e1 ± 9.35e-1 (n=2) | 1.570e1 ± 6.08e0 (n=6) | 1.636e1 ± 4.91e0 (n=32) | 1.866e1 ± 4.20e0 (n=8) | 2.954e1 ± 6.13e0 (n=2) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| ModifiedCosine (mass-spectrometry-traits) | 1.262e1 ± 2.46e0 (n=2) | 3.199e1 ± 1.59e1 (n=6) | 3.612e1 ± 1.72e1 (n=32) | 4.565e1 ± 1.08e1 (n=8) | 1.103e2 ± 2.24e1 (n=2) |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 1.259e1 ± 2.49e0 (n=2) | 2.945e1 ± 1.39e1 (n=6) | 3.278e1 ± 1.22e1 (n=32) | 4.476e1 ± 1.11e1 (n=8) | 1.078e2 ± 2.02e1 (n=2) |
| ModifiedGreedyCosine (matchms) | 2.603e1 ± 3.54e-2 (n=2) | 3.591e1 ± 8.16e0 (n=6) | 3.646e1 ± 7.50e0 (n=32) | 3.763e1 ± 6.87e0 (n=8) | 4.840e1 ± 3.05e0 (n=2) |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.260e1 ± 2.66e0 (n=2) | 2.862e1 ± 1.42e1 (n=6) | 3.031e1 ± 1.48e1 (n=32) | 3.870e1 ± 1.61e1 (n=8) | 9.807e1 ± 7.13e0 (n=2) |

## RMSE vs Reference by Peak Count (Spectra used: 66)

Y-axis: `RMSE`

### Reference: CosineHungarian (matchms)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 1.000e-16 (n=2) | 1.000e-16 ± 2.90e-25 (n=6) | 1.000e-16 (n=32) | 1.000e-16 ± 3.47e-25 (n=8) | 1.000e-16 (n=2) |
| CosineGreedy (matchms) | 1.000e-16 (n=2) | 1.000e-16 ± 2.90e-25 (n=6) | 1.000e-16 (n=32) | 1.000e-16 ± 3.47e-25 (n=8) | 1.000e-16 (n=2) |
| CosineHungarian (mass-spectrometry-traits) | 1.000e-16 (n=2) | 1.000e-16 ± 2.90e-25 (n=6) | 1.000e-16 (n=32) | 1.000e-16 ± 3.47e-25 (n=8) | 1.000e-16 (n=2) |
| LinearCosine (mass-spectrometry-traits) | 1.000e-16 (n=2) | 2.554e-8 ± 2.37e-8 (n=6) | 1.086e-5 ± 1.11e-5 (n=32) | 5.107e-5 ± 5.00e-5 (n=8) | 1.000e-16 (n=2) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 1.000e-16 (n=2) | 1.000e-16 ± 2.90e-25 (n=6) | 1.879e-9 ± 2.03e-9 (n=32) | 8.847e-10 ± 8.66e-10 (n=8) | 1.000e-16 (n=2) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.000e-16 (n=2) | 1.000e-16 ± 2.90e-25 (n=6) | 1.668e-9 ± 2.00e-9 (n=32) | 1.695e-9 ± 1.66e-9 (n=8) | 1.000e-16 (n=2) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 |
| --- | --- | --- | --- | --- | --- |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 1.000e-16 (n=2) | 1.000e-16 ± 2.90e-25 (n=6) | 1.004e-16 ± 1.20e-18 (n=32) | 1.000e-16 ± 3.47e-25 (n=8) | 1.000e-16 (n=2) |
| ModifiedGreedyCosine (matchms) | 1.000e-16 (n=2) | 1.000e-16 ± 2.90e-25 (n=6) | 1.004e-16 ± 1.20e-18 (n=32) | 1.000e-16 ± 3.47e-25 (n=8) | 1.000e-16 (n=2) |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.000e-16 (n=2) | 2.554e-8 ± 2.37e-8 (n=6) | 5.027e-4 ± 5.57e-4 (n=32) | 7.694e-5 ± 6.61e-5 (n=8) | 1.000e-16 (n=2) |

