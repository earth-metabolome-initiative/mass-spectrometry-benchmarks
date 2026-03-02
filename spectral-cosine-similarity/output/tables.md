# Benchmark Tables

## Run Scope

- Requested max spectra: `200`
- Total spectra in DB: `200`
- Spectra used in results: `200`

## Timing by Peak Count (Spectra used: 200)

Y-axis: `Mean time (µs)`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 2.559e0 ± 2.28e0 (n=2388) | 2.914e0 ± 2.47e0 (n=9192) | 3.420e0 ± 2.58e0 (n=12708) | 4.581e0 ± 2.78e0 (n=32132) | 7.158e0 ± 3.42e0 (n=19276) | 1.338e1 ± 5.09e0 (n=4020) | 2.586e1 ± 1.12e1 (n=684) |
| CosineGreedy (matchms) | 2.128e1 ± 6.14e0 (n=2388) | 2.488e1 ± 7.91e0 (n=9192) | 2.831e1 ± 8.69e0 (n=12708) | 3.281e1 ± 1.20e1 (n=32132) | 4.192e1 ± 1.75e1 (n=19276) | 6.798e1 ± 4.39e1 (n=4020) | 1.378e2 ± 1.15e2 (n=684) |
| CosineHungarian (mass-spectrometry-traits) | 2.588e0 ± 2.28e0 (n=2388) | 2.989e0 ± 2.47e0 (n=9192) | 3.744e0 ± 2.90e0 (n=12708) | 5.571e0 ± 4.48e0 (n=32132) | 1.093e1 ± 1.07e1 (n=19276) | 3.766e1 ± 4.86e1 (n=4020) | 1.625e2 ± 2.09e2 (n=684) |
| CosineHungarian (matchms) | 2.941e1 ± 1.91e1 (n=2388) | 3.999e1 ± 2.32e1 (n=9192) | 5.378e1 ± 3.00e1 (n=12708) | 7.155e1 ± 6.10e1 (n=32132) | 1.278e2 ± 1.64e2 (n=19276) | 4.381e2 ± 7.11e2 (n=4020) | 1.897e3 ± 2.93e3 (n=684) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 5.012e-1 ± 4.29e-1 (n=2388) | 5.533e-1 ± 4.63e-1 (n=9192) | 6.504e-1 ± 5.03e-1 (n=12708) | 8.394e-1 ± 5.40e-1 (n=32132) | 1.305e0 ± 6.68e-1 (n=19276) | 2.615e0 ± 1.00e0 (n=4020) | 5.038e0 ± 1.88e0 (n=684) |
| EntropySimilarityUnweighted (ms_entropy) | 1.157e1 ± 8.84e-1 (n=2388) | 1.166e1 ± 1.08e0 (n=9192) | 1.178e1 ± 1.12e0 (n=12708) | 1.189e1 ± 9.78e-1 (n=32132) | 1.243e1 ± 1.08e0 (n=19276) | 1.337e1 ± 1.32e0 (n=4020) | 1.502e1 ± 1.79e0 (n=684) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.707e0 ± 1.06e0 (n=2388) | 1.919e0 ± 1.11e0 (n=9192) | 2.190e0 ± 1.08e0 (n=12708) | 2.956e0 ± 1.21e0 (n=32132) | 4.130e0 ± 1.48e0 (n=19276) | 6.102e0 ± 1.88e0 (n=4020) | 8.670e0 ± 2.26e0 (n=684) |
| EntropySimilarityWeighted (ms_entropy) | 1.334e1 ± 1.38e0 (n=2388) | 1.353e1 ± 1.41e0 (n=9192) | 1.376e1 ± 1.38e0 (n=12708) | 1.432e1 ± 1.52e0 (n=32132) | 1.523e1 ± 1.49e0 (n=19276) | 1.718e1 ± 2.16e0 (n=4020) | 1.944e1 ± 1.98e0 (n=684) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosine (mass-spectrometry-traits) | 3.089e0 ± 2.58e0 (n=2388) | 3.562e0 ± 2.88e0 (n=9192) | 4.616e0 ± 3.69e0 (n=12708) | 7.685e0 ± 6.99e0 (n=32132) | 1.734e1 ± 1.90e1 (n=19276) | 6.858e1 ± 7.95e1 (n=4020) | 2.797e2 ± 3.19e2 (n=684) |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 2.882e0 ± 2.47e0 (n=2388) | 3.236e0 ± 2.64e0 (n=9192) | 3.942e0 ± 2.91e0 (n=12708) | 5.386e0 ± 3.27e0 (n=32132) | 8.661e0 ± 4.42e0 (n=19276) | 1.734e1 ± 8.16e0 (n=4020) | 3.458e1 ± 1.94e1 (n=684) |
| ModifiedGreedyCosine (matchms) | 3.253e1 ± 7.11e0 (n=2388) | 3.467e1 ± 7.64e0 (n=9192) | 3.847e1 ± 8.73e0 (n=12708) | 4.429e1 ± 1.35e1 (n=32132) | 5.681e1 ± 2.61e1 (n=19276) | 1.014e2 ± 7.38e1 (n=4020) | 2.184e2 ± 2.05e2 (n=684) |

## RMSE vs Reference by Peak Count (Spectra used: 200)

Y-axis: `RMSE`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (mass-spectrometry-traits) | 3.629e-5 ± 1.19e-4 (n=2388) | 3.023e-4 ± 1.27e-3 (n=9192) | 5.717e-5 ± 2.05e-4 (n=12708) | 8.722e-5 ± 4.36e-4 (n=32132) | 3.357e-4 ± 1.36e-3 (n=19276) | 2.838e-4 ± 7.14e-4 (n=4020) | 3.457e-4 ± 6.96e-4 (n=684) |
| CosineGreedy (matchms) | 3.629e-5 ± 1.19e-4 (n=2388) | 3.023e-4 ± 1.27e-3 (n=9192) | 5.717e-5 ± 2.05e-4 (n=12708) | 8.721e-5 ± 4.36e-4 (n=32132) | 3.357e-4 ± 1.36e-3 (n=19276) | 2.838e-4 ± 7.14e-4 (n=4020) | 3.457e-4 ± 6.96e-4 (n=684) |
| CosineHungarian (mass-spectrometry-traits) | 1.000e-16 ± 1.73e-19 (n=2388) | 1.003e-16 ± 1.05e-17 (n=9192) | 1.007e-16 ± 8.67e-18 (n=12708) | 1.010e-16 ± 1.03e-17 (n=32132) | 1.018e-16 ± 1.78e-17 (n=19276) | 1.045e-16 ± 2.73e-17 (n=4020) | 1.479e-16 ± 1.02e-16 (n=684) |

### Reference: EntropySimilarityUnweighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.443e-9 ± 7.42e-9 (n=2388) | 9.079e-9 ± 1.09e-8 (n=9192) | 1.142e-8 ± 1.41e-8 (n=12708) | 1.208e-8 ± 2.08e-8 (n=32132) | 1.845e-8 ± 3.02e-8 (n=19276) | 2.714e-8 ± 3.33e-8 (n=4020) | 9.045e-8 ± 1.10e-7 (n=684) |

### Reference: EntropySimilarityWeighted (ms_entropy)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.012e-8 ± 1.54e-8 (n=2388) | 1.259e-8 ± 2.03e-8 (n=9192) | 1.275e-8 ± 1.33e-8 (n=12708) | 1.708e-8 ± 2.59e-8 (n=32132) | 2.204e-8 ± 3.16e-8 (n=19276) | 3.238e-8 ± 4.05e-8 (n=4020) | 9.045e-8 ± 1.10e-7 (n=684) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedGreedyCosine (mass-spectrometry-traits) | 2.459e-4 ± 6.72e-4 (n=2388) | 4.351e-4 ± 1.70e-3 (n=9192) | 5.024e-4 ± 1.94e-3 (n=12708) | 4.098e-4 ± 2.14e-3 (n=32132) | 1.630e-3 ± 7.28e-3 (n=19276) | 7.045e-4 ± 1.26e-3 (n=4020) | 1.691e-3 ± 2.48e-3 (n=684) |
| ModifiedGreedyCosine (matchms) | 2.459e-4 ± 6.72e-4 (n=2388) | 4.351e-4 ± 1.70e-3 (n=9192) | 5.024e-4 ± 1.94e-3 (n=12708) | 4.098e-4 ± 2.14e-3 (n=32132) | 1.630e-3 ± 7.28e-3 (n=19276) | 7.045e-4 ± 1.26e-3 (n=4020) | 1.691e-3 ± 2.48e-3 (n=684) |

