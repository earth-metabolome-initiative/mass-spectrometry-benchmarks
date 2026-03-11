# Benchmark Tables

## Run Scope

- Total spectra in DB: `199032`
- Spectra used in results: `199032`

## Timing by Peak Count (Spectra used: 199032, Pairs: 10000000)

Y-axis: `Mean time (µs)`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (matchms) | 1.743e1 ± 3.12e0 (n=2814384) | 1.853e1 ± 4.17e0 (n=3379884) | 2.078e1 ± 5.38e0 (n=2557990) | 2.427e1 ± 6.26e0 (n=970458) | 2.869e1 ± 8.92e0 (n=238279) | 3.457e1 ± 1.00e1 (n=38252) | 4.355e1 ± 8.80e0 (n=753) |
| CosineHungarian (mass-spectrometry-traits) | 1.139e0 ± 1.02e0 (n=2814384) | 1.475e0 ± 1.09e0 (n=3379884) | 2.162e0 ± 1.23e0 (n=2557990) | 3.443e0 ± 1.50e0 (n=970458) | 6.276e0 ± 4.45e0 (n=238279) | 1.024e1 ± 7.34e0 (n=38252) | 1.568e1 ± 8.96e0 (n=753) |
| CosineHungarian (matchms) | 2.683e1 ± 1.28e1 (n=2814384) | 3.117e1 ± 1.70e1 (n=3379884) | 3.992e1 ± 2.14e1 (n=2557990) | 5.206e1 ± 2.42e1 (n=970458) | 6.976e1 ± 5.80e1 (n=238279) | 8.871e1 ± 8.68e1 (n=38252) | 1.218e2 ± 1.10e2 (n=753) |

### Reference: CosineHungarianMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarianMerged (mass-spectrometry-traits) | 1.027e0 ± 9.10e-1 (n=2814384) | 1.326e0 ± 9.82e-1 (n=3379884) | 1.938e0 ± 1.11e0 (n=2557990) | 3.051e0 ± 1.35e0 (n=970458) | 4.969e0 ± 1.87e0 (n=238279) | 8.366e0 ± 3.22e0 (n=38252) | 1.428e1 ± 8.84e0 (n=753) |
| LinearCosine (mass-spectrometry-traits) | 1.004e0 ± 9.12e-1 (n=2814384) | 1.266e0 ± 9.69e-1 (n=3379884) | 1.770e0 ± 1.07e0 (n=2557990) | 2.707e0 ± 1.23e0 (n=970458) | 4.283e0 ± 1.43e0 (n=238279) | 7.019e0 ± 1.67e0 (n=38252) | 1.066e1 ± 1.05e0 (n=753) |

### Reference: EntropySimilarityUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 4.535e-1 ± 3.13e-1 (n=2814384) | 5.501e-1 ± 3.40e-1 (n=3379884) | 7.279e-1 ± 4.01e-1 (n=2557990) | 1.022e0 ± 5.30e-1 (n=970458) | 1.281e0 ± 7.60e-1 (n=238279) | 1.474e0 ± 1.02e0 (n=38252) | 1.909e0 ± 1.19e0 (n=753) |
| EntropySimilarityUnweighted (ms_entropy) | 9.516e0 ± 3.03e0 (n=2814384) | 1.022e1 ± 3.24e0 (n=3379884) | 1.172e1 ± 3.61e0 (n=2557990) | 1.474e1 ± 4.11e0 (n=970458) | 2.025e1 ± 4.53e0 (n=238279) | 2.969e1 ± 5.19e0 (n=38252) | 4.282e1 ± 3.53e0 (n=753) |

### Reference: EntropySimilarityWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 8.263e-1 ± 4.65e-1 (n=2814384) | 1.011e0 ± 5.00e-1 (n=3379884) | 1.334e0 ± 5.88e-1 (n=2557990) | 1.762e0 ± 7.43e-1 (n=970458) | 2.041e0 ± 1.01e0 (n=238279) | 2.293e0 ± 1.34e0 (n=38252) | 2.908e0 ± 1.57e0 (n=753) |
| EntropySimilarityWeighted (ms_entropy) | 1.013e1 ± 3.07e0 (n=2814384) | 1.089e1 ± 3.27e0 (n=3379884) | 1.246e1 ± 3.63e0 (n=2557990) | 1.556e1 ± 4.13e0 (n=970458) | 2.109e1 ± 4.56e0 (n=238279) | 3.055e1 ± 5.21e0 (n=38252) | 4.366e1 ± 3.42e0 (n=753) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosine (mass-spectrometry-traits) | 1.222e0 ± 1.05e0 (n=2814384) | 1.575e0 ± 1.14e0 (n=3379884) | 2.273e0 ± 1.29e0 (n=2557990) | 3.663e0 ± 1.60e0 (n=970458) | 6.786e0 ± 4.58e0 (n=238279) | 1.138e1 ± 7.92e0 (n=38252) | 1.817e1 ± 1.09e1 (n=753) |
| ModifiedCosineHungarian (matchms) | 3.601e1 ± 1.80e1 (n=2814384) | 4.059e1 ± 1.93e1 (n=3379884) | 4.843e1 ± 2.02e1 (n=2557990) | 5.965e1 ± 2.25e1 (n=970458) | 7.860e1 ± 5.00e1 (n=238279) | 1.005e2 ± 6.84e1 (n=38252) | 1.427e2 ± 9.42e1 (n=753) |
| ModifiedGreedyCosine (matchms) | 2.577e1 ± 5.15e0 (n=2814384) | 2.721e1 ± 5.61e0 (n=3379884) | 2.960e1 ± 5.94e0 (n=2557990) | 3.300e1 ± 6.16e0 (n=970458) | 3.792e1 ± 8.59e0 (n=238279) | 4.452e1 ± 9.91e0 (n=38252) | 5.396e1 ± 1.10e1 (n=753) |

### Reference: ModifiedCosineMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosineMerged (mass-spectrometry-traits) | 1.170e0 ± 1.01e0 (n=2814384) | 1.501e0 ± 1.09e0 (n=3379884) | 2.155e0 ± 1.25e0 (n=2557990) | 3.429e0 ± 1.57e0 (n=970458) | 5.723e0 ± 2.31e0 (n=238279) | 9.884e0 ± 4.67e0 (n=38252) | 1.752e1 ± 1.13e1 (n=753) |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.153e0 ± 9.88e-1 (n=2814384) | 1.459e0 ± 1.07e0 (n=3379884) | 2.058e0 ± 1.21e0 (n=2557990) | 3.209e0 ± 1.47e0 (n=970458) | 5.203e0 ± 1.89e0 (n=238279) | 8.684e0 ± 2.65e0 (n=38252) | 1.385e1 ± 3.30e0 (n=753) |

### Reference: ModifiedLinearEntropyUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.570e-1 ± 3.53e-1 (n=2814384) | 6.725e-1 ± 3.91e-1 (n=3379884) | 8.899e-1 ± 4.83e-1 (n=2557990) | 1.256e0 ± 6.88e-1 (n=970458) | 1.576e0 ± 1.04e0 (n=238279) | 1.826e0 ± 1.45e0 (n=38252) | 2.398e0 ± 1.74e0 (n=753) |

### Reference: ModifiedLinearEntropyWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 9.130e-1 ± 4.90e-1 (n=2814384) | 1.114e0 ± 5.34e-1 (n=3379884) | 1.469e0 ± 6.50e-1 (n=2557990) | 1.959e0 ± 8.71e-1 (n=970458) | 2.295e0 ± 1.26e0 (n=238279) | 2.599e0 ± 1.71e0 (n=38252) | 3.347e0 ± 2.07e0 (n=753) |

## RMSE vs Reference by Peak Count (Spectra used: 199032, Pairs: 10000000)

Y-axis: `RMSE`

### Reference: CosineHungarian (matchms)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| CosineGreedy (matchms) | 1.007e-4 ± 1.40e-3 (n=2814384) | 1.574e-5 ± 2.46e-4 (n=3379884) | 1.638e-5 ± 2.31e-4 (n=2557990) | 1.580e-5 ± 1.78e-4 (n=970458) | 6.825e-6 ± 4.52e-5 (n=238279) | 3.501e-6 ± 7.11e-6 (n=38252) | 7.073e-8 ± 1.89e-7 (n=753) |
| CosineHungarian (mass-spectrometry-traits) | 1.009e-16 ± 1.14e-17 (n=2814384) | 1.051e-16 ± 3.84e-17 (n=3379884) | 1.321e-16 ± 1.25e-16 (n=2557990) | 2.769e-16 ± 4.71e-16 (n=970458) | 1.159e-15 ± 2.00e-15 (n=238279) | 1.409e-15 ± 2.26e-15 (n=38252) | 1.758e-15 ± 2.41e-15 (n=753) |

### Reference: CosineHungarianMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| LinearCosine (mass-spectrometry-traits) | 1.000e-16 ± 1.83e-22 (n=2814384) | 1.000e-16 (n=3379884) | 1.000e-16 ± 2.27e-22 (n=2557990) | 1.000e-16 (n=970458) | 1.000e-16 ± 4.56e-23 (n=238279) | 1.000e-16 ± 1.96e-23 (n=38252) | 1.000e-16 (n=753) |

### Reference: EntropySimilarityUnweighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityUnweighted (ms_entropy) | 4.832e-4 ± 5.48e-3 (n=2814384) | 7.721e-4 ± 1.13e-2 (n=3379884) | 5.813e-4 ± 6.45e-3 (n=2557990) | 4.277e-4 ± 3.12e-3 (n=970458) | 6.410e-4 ± 4.76e-3 (n=238279) | 5.052e-4 ± 2.84e-3 (n=38252) | 1.418e-4 ± 3.20e-4 (n=753) |

### Reference: EntropySimilarityWeighted (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| EntropySimilarityWeighted (ms_entropy) | 5.400e-4 ± 5.03e-3 (n=2814384) | 6.214e-4 ± 6.95e-3 (n=3379884) | 5.797e-4 ± 4.87e-3 (n=2557990) | 4.760e-4 ± 3.04e-3 (n=970458) | 9.303e-4 ± 5.58e-3 (n=238279) | 5.894e-4 ± 3.02e-3 (n=38252) | 1.418e-4 ± 3.20e-4 (n=753) |

### Reference: ModifiedCosine (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedCosineHungarian (matchms) | 1.241e-3 ± 1.58e-2 (n=2814384) | 7.742e-4 ± 1.23e-2 (n=3379884) | 1.009e-3 ± 1.28e-2 (n=2557990) | 1.550e-4 ± 1.87e-3 (n=970458) | 2.341e-5 ± 2.08e-4 (n=238279) | 7.866e-5 ± 5.51e-4 (n=38252) | 1.456e-9 ± 3.88e-9 (n=753) |
| ModifiedGreedyCosine (matchms) | 1.249e-3 ± 1.58e-2 (n=2814384) | 8.150e-4 ± 1.23e-2 (n=3379884) | 1.028e-3 ± 1.28e-2 (n=2557990) | 2.082e-4 ± 1.94e-3 (n=970458) | 2.486e-4 ± 2.35e-3 (n=238279) | 1.290e-4 ± 6.08e-4 (n=38252) | 2.478e-4 ± 6.43e-4 (n=753) |

### Reference: ModifiedCosineMerged (mass-spectrometry-traits)

| Series | 5–8 | 9–16 | 17–32 | 33–64 | 65–128 | 129–256 | 257–512 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| ModifiedLinearCosine (mass-spectrometry-traits) | 1.000e-16 ± 2.64e-19 (n=2814384) | 1.000e-16 ± 4.05e-19 (n=3379884) | 1.000e-16 ± 5.00e-19 (n=2557990) | 1.000e-16 ± 1.03e-18 (n=970458) | 1.001e-16 ± 2.14e-18 (n=238279) | 1.002e-16 ± 3.76e-18 (n=38252) | 1.000e-16 ± 2.19e-19 (n=753) |

## Spectral Similarity vs Structural Similarity (Tanimoto) (Spectra used: 199032, Pairs: 10000000)

Y-axis: `Mean spectral similarity`

### atompair

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 3.710e-3 ± 3.03e-2 (n=2138540) | 5.387e-3 ± 3.57e-2 (n=3954911) | 6.132e-3 ± 3.86e-2 (n=2888032) | 7.226e-3 ± 4.57e-2 (n=813299) | 2.149e-2 ± 1.08e-1 (n=136582) | 1.062e-1 ± 2.74e-1 (n=34985) | 2.094e-1 ± 3.79e-1 (n=18614) | 3.152e-1 ± 4.43e-1 (n=8695) | 3.720e-1 ± 4.62e-1 (n=3340) | 4.519e-1 ± 4.23e-1 (n=3002) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 5.527e-3 ± 2.87e-2 (n=2138540) | 8.260e-3 ± 3.49e-2 (n=3954911) | 9.581e-3 ± 3.82e-2 (n=2888032) | 1.109e-2 ± 4.49e-2 (n=813299) | 2.514e-2 ± 9.57e-2 (n=136582) | 1.004e-1 ± 2.38e-1 (n=34985) | 1.961e-1 ± 3.40e-1 (n=18614) | 3.023e-1 ± 4.05e-1 (n=8695) | 3.553e-1 ± 4.23e-1 (n=3340) | 4.511e-1 ± 3.90e-1 (n=3002) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 6.974e-3 ± 3.07e-2 (n=2138540) | 1.010e-2 ± 3.71e-2 (n=3954911) | 1.151e-2 ± 3.99e-2 (n=2888032) | 1.309e-2 ± 4.58e-2 (n=813299) | 2.602e-2 ± 8.24e-2 (n=136582) | 8.395e-2 ± 1.78e-1 (n=34985) | 1.555e-1 ± 2.51e-1 (n=18614) | 2.361e-1 ± 3.00e-1 (n=8695) | 2.778e-1 ± 3.13e-1 (n=3340) | 4.318e-1 ± 3.51e-1 (n=3002) |
| ModifiedCosine (mass-spectrometry-traits) | 5.833e-2 ± 1.72e-1 (n=2138540) | 5.349e-2 ± 1.65e-1 (n=3954911) | 5.455e-2 ± 1.67e-1 (n=2888032) | 5.777e-2 ± 1.74e-1 (n=813299) | 8.162e-2 ± 2.12e-1 (n=136582) | 1.703e-1 ± 3.20e-1 (n=34985) | 2.703e-1 ± 4.00e-1 (n=18614) | 3.702e-1 ± 4.48e-1 (n=8695) | 4.342e-1 ± 4.63e-1 (n=3340) | 4.787e-1 ± 4.17e-1 (n=3002) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.571e-2 ± 1.37e-1 (n=2138540) | 5.207e-2 ± 1.31e-1 (n=3954911) | 5.338e-2 ± 1.33e-1 (n=2888032) | 5.633e-2 ± 1.39e-1 (n=813299) | 7.767e-2 ± 1.73e-1 (n=136582) | 1.572e-1 ± 2.70e-1 (n=34985) | 2.501e-1 ± 3.54e-1 (n=18614) | 3.494e-1 ± 4.08e-1 (n=8695) | 4.092e-1 ± 4.22e-1 (n=3340) | 4.730e-1 ± 3.83e-1 (n=3002) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 5.526e-2 ± 1.10e-1 (n=2138540) | 5.151e-2 ± 1.05e-1 (n=3954911) | 5.259e-2 ± 1.06e-1 (n=2888032) | 5.503e-2 ± 1.12e-1 (n=813299) | 7.338e-2 ± 1.41e-1 (n=136582) | 1.353e-1 ± 2.07e-1 (n=34985) | 2.047e-1 ± 2.69e-1 (n=18614) | 2.802e-1 ± 3.10e-1 (n=8695) | 3.295e-1 ± 3.23e-1 (n=3340) | 4.509e-1 ± 3.42e-1 (n=3002) |

### ecfp

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 4.063e-3 ± 2.95e-2 (n=5907243) | 6.716e-3 ± 4.08e-2 (n=3728780) | 1.540e-2 ± 7.43e-2 (n=261771) | 3.048e-2 ± 1.18e-1 (n=46007) | 6.410e-2 ± 1.87e-1 (n=17266) | 2.227e-1 ± 3.80e-1 (n=13924) | 2.930e-1 ± 4.07e-1 (n=7976) | 3.893e-1 ± 4.48e-1 (n=4886) | 4.770e-1 ± 4.83e-1 (n=4402) | 4.763e-1 ± 4.66e-1 (n=7745) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.385e-3 ± 2.94e-2 (n=5907243) | 1.038e-2 ± 4.02e-2 (n=3728780) | 2.078e-2 ± 6.79e-2 (n=261771) | 3.666e-2 ± 1.03e-1 (n=46007) | 6.843e-2 ± 1.61e-1 (n=17266) | 2.081e-1 ± 3.37e-1 (n=13924) | 2.633e-1 ± 3.45e-1 (n=7976) | 3.549e-1 ± 3.89e-1 (n=4886) | 4.424e-1 ± 4.39e-1 (n=4402) | 4.574e-1 ± 4.30e-1 (n=7745) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.800e-3 ± 3.15e-2 (n=5907243) | 1.263e-2 ± 4.23e-2 (n=3728780) | 2.462e-2 ± 6.72e-2 (n=261771) | 4.088e-2 ± 9.64e-2 (n=46007) | 6.999e-2 ± 1.41e-1 (n=17266) | 1.680e-1 ± 2.50e-1 (n=13924) | 2.081e-1 ± 2.53e-1 (n=7976) | 2.727e-1 ± 2.80e-1 (n=4886) | 3.302e-1 ± 3.21e-1 (n=4402) | 3.773e-1 ± 3.42e-1 (n=7745) |
| ModifiedCosine (mass-spectrometry-traits) | 5.249e-2 ± 1.65e-1 (n=5907243) | 5.805e-2 ± 1.71e-1 (n=3728780) | 7.764e-2 ± 1.96e-1 (n=261771) | 1.047e-1 ± 2.28e-1 (n=46007) | 1.513e-1 ± 2.77e-1 (n=17266) | 2.806e-1 ± 3.96e-1 (n=13924) | 3.489e-1 ± 4.15e-1 (n=7976) | 4.366e-1 ± 4.47e-1 (n=4886) | 5.108e-1 ± 4.77e-1 (n=4402) | 4.884e-1 ± 4.64e-1 (n=7745) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.010e-2 ± 1.30e-1 (n=5907243) | 5.738e-2 ± 1.37e-1 (n=3728780) | 7.853e-2 ± 1.60e-1 (n=261771) | 1.047e-1 ± 1.88e-1 (n=46007) | 1.479e-1 ± 2.32e-1 (n=17266) | 2.609e-1 ± 3.47e-1 (n=13924) | 3.153e-1 ± 3.54e-1 (n=7976) | 3.985e-1 ± 3.88e-1 (n=4886) | 4.709e-1 ± 4.35e-1 (n=4402) | 4.670e-1 ± 4.29e-1 (n=7745) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 4.896e-2 ± 1.03e-1 (n=5907243) | 5.722e-2 ± 1.10e-1 (n=3728780) | 7.981e-2 ± 1.34e-1 (n=261771) | 1.050e-1 ± 1.61e-1 (n=46007) | 1.441e-1 ± 1.98e-1 (n=17266) | 2.172e-1 ± 2.64e-1 (n=13924) | 2.589e-1 ± 2.71e-1 (n=7976) | 3.158e-1 ± 2.88e-1 (n=4886) | 3.577e-1 ± 3.23e-1 (n=4402) | 3.871e-1 ± 3.42e-1 (n=7745) |

### fcfp

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 3.930e-3 ± 2.91e-2 (n=3915438) | 5.316e-3 ± 3.52e-2 (n=4915446) | 9.917e-3 ± 5.34e-2 (n=945003) | 1.931e-2 ± 8.38e-2 (n=131708) | 6.077e-2 ± 1.98e-1 (n=42185) | 1.210e-1 ± 2.86e-1 (n=18885) | 1.627e-1 ± 3.16e-1 (n=9037) | 3.212e-1 ± 4.20e-1 (n=7258) | 3.768e-1 ± 4.60e-1 (n=4859) | 4.713e-1 ± 4.70e-1 (n=10181) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.057e-3 ± 2.84e-2 (n=3915438) | 8.411e-3 ± 3.50e-2 (n=4915446) | 1.462e-2 ± 5.17e-2 (n=945003) | 2.541e-2 ± 7.65e-2 (n=131708) | 6.264e-2 ± 1.75e-1 (n=42185) | 1.207e-1 ± 2.56e-1 (n=18885) | 1.531e-1 ± 2.62e-1 (n=9037) | 2.883e-1 ± 3.56e-1 (n=7258) | 3.556e-1 ± 4.15e-1 (n=4859) | 4.506e-1 ± 4.33e-1 (n=10181) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.428e-3 ± 3.05e-2 (n=3915438) | 1.028e-2 ± 3.71e-2 (n=4915446) | 1.747e-2 ± 5.29e-2 (n=945003) | 2.940e-2 ± 7.51e-2 (n=131708) | 5.900e-2 ± 1.38e-1 (n=42185) | 1.069e-1 ± 1.98e-1 (n=18885) | 1.339e-1 ± 2.03e-1 (n=9037) | 2.255e-1 ± 2.58e-1 (n=7258) | 2.751e-1 ± 3.02e-1 (n=4859) | 3.653e-1 ± 3.39e-1 (n=10181) |
| ModifiedCosine (mass-spectrometry-traits) | 5.469e-2 ± 1.69e-1 (n=3915438) | 5.306e-2 ± 1.64e-1 (n=4915446) | 6.404e-2 ± 1.78e-1 (n=945003) | 8.826e-2 ± 2.06e-1 (n=131708) | 1.375e-1 ± 2.73e-1 (n=42185) | 2.083e-1 ± 3.37e-1 (n=18885) | 2.560e-1 ± 3.57e-1 (n=9037) | 3.876e-1 ± 4.24e-1 (n=7258) | 4.394e-1 ± 4.58e-1 (n=4859) | 4.923e-1 ± 4.67e-1 (n=10181) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.156e-2 ± 1.33e-1 (n=3915438) | 5.224e-2 ± 1.31e-1 (n=4915446) | 6.413e-2 ± 1.43e-1 (n=945003) | 8.828e-2 ± 1.68e-1 (n=131708) | 1.316e-1 ± 2.29e-1 (n=42185) | 1.999e-1 ± 2.91e-1 (n=18885) | 2.397e-1 ± 2.95e-1 (n=9037) | 3.490e-1 ± 3.60e-1 (n=7258) | 4.112e-1 ± 4.11e-1 (n=4859) | 4.682e-1 ± 4.30e-1 (n=10181) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 5.013e-2 ± 1.05e-1 (n=3915438) | 5.188e-2 ± 1.05e-1 (n=4915446) | 6.446e-2 ± 1.17e-1 (n=945003) | 8.883e-2 ± 1.40e-1 (n=131708) | 1.229e-1 ± 1.83e-1 (n=42185) | 1.807e-1 ± 2.29e-1 (n=18885) | 2.155e-1 ± 2.36e-1 (n=9037) | 2.835e-1 ± 2.71e-1 (n=7258) | 3.270e-1 ± 3.07e-1 (n=4859) | 3.832e-1 ± 3.39e-1 (n=10181) |

### maccs

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 4.338e-3 ± 3.22e-2 (n=223519) | 4.562e-3 ± 3.29e-2 (n=1447496) | 4.839e-3 ± 3.36e-2 (n=2776961) | 5.212e-3 ± 3.47e-2 (n=2712721) | 5.693e-3 ± 3.75e-2 (n=1677119) | 7.218e-3 ± 4.44e-2 (n=758877) | 1.228e-2 ± 6.69e-2 (n=251213) | 4.861e-2 ± 1.82e-1 (n=89511) | 4.086e-2 ± 1.45e-1 (n=29286) | 2.976e-1 ± 4.28e-1 (n=33297) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.282e-3 ± 3.05e-2 (n=223519) | 6.910e-3 ± 3.15e-2 (n=1447496) | 7.514e-3 ± 3.30e-2 (n=2776961) | 8.129e-3 ± 3.43e-2 (n=2712721) | 8.910e-3 ± 3.72e-2 (n=1677119) | 1.101e-2 ± 4.39e-2 (n=758877) | 1.650e-2 ± 6.13e-2 (n=251213) | 5.071e-2 ± 1.64e-1 (n=89511) | 4.695e-2 ± 1.28e-1 (n=29286) | 2.779e-1 ± 3.84e-1 (n=33297) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 7.810e-3 ± 3.28e-2 (n=223519) | 8.590e-3 ± 3.38e-2 (n=1447496) | 9.248e-3 ± 3.52e-2 (n=2776961) | 9.888e-3 ± 3.64e-2 (n=2712721) | 1.075e-2 ± 3.89e-2 (n=1677119) | 1.311e-2 ± 4.52e-2 (n=758877) | 1.873e-2 ± 5.96e-2 (n=251213) | 4.671e-2 ± 1.29e-1 (n=89511) | 5.047e-2 ± 1.17e-1 (n=29286) | 2.214e-1 ± 2.92e-1 (n=33297) |
| ModifiedCosine (mass-spectrometry-traits) | 5.818e-2 ± 1.67e-1 (n=223519) | 5.445e-2 ± 1.64e-1 (n=1447496) | 5.228e-2 ± 1.62e-1 (n=2776961) | 5.301e-2 ± 1.65e-1 (n=2712721) | 5.633e-2 ± 1.71e-1 (n=1677119) | 6.344e-2 ± 1.82e-1 (n=758877) | 7.671e-2 ± 2.00e-1 (n=251213) | 1.174e-1 ± 2.59e-1 (n=89511) | 1.387e-1 ± 2.68e-1 (n=29286) | 3.559e-1 ± 4.34e-1 (n=33297) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.449e-2 ± 1.32e-1 (n=223519) | 5.274e-2 ± 1.31e-1 (n=1447496) | 5.114e-2 ± 1.29e-1 (n=2776961) | 5.167e-2 ± 1.31e-1 (n=2712721) | 5.466e-2 ± 1.36e-1 (n=1677119) | 6.097e-2 ± 1.45e-1 (n=758877) | 7.261e-2 ± 1.59e-1 (n=251213) | 1.104e-1 ± 2.17e-1 (n=89511) | 1.328e-1 ± 2.22e-1 (n=29286) | 3.298e-1 ± 3.86e-1 (n=33297) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 5.373e-2 ± 1.06e-1 (n=223519) | 5.268e-2 ± 1.05e-1 (n=1447496) | 5.103e-2 ± 1.04e-1 (n=2776961) | 5.103e-2 ± 1.05e-1 (n=2712721) | 5.349e-2 ± 1.09e-1 (n=1677119) | 5.912e-2 ± 1.15e-1 (n=758877) | 6.922e-2 ± 1.27e-1 (n=251213) | 9.980e-2 ± 1.69e-1 (n=89511) | 1.269e-1 ± 1.87e-1 (n=29286) | 2.696e-1 ± 2.98e-1 (n=33297) |

### map

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 6.058e-3 ± 3.85e-2 (n=4945110) | 5.390e-3 ± 3.65e-2 (n=3730244) | 4.353e-3 ± 3.70e-2 (n=949125) | 7.912e-3 ± 6.75e-2 (n=243342) | 4.331e-2 ± 1.89e-1 (n=89309) | 1.025e-1 ± 2.80e-1 (n=26575) | 3.031e-1 ± 4.45e-1 (n=8494) | 4.242e-1 ± 4.81e-1 (n=3946) | 4.390e-1 ± 4.84e-1 (n=1205) | 4.614e-1 ± 4.15e-1 (n=2650) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 9.232e-3 ± 3.76e-2 (n=4945110) | 8.324e-3 ± 3.57e-2 (n=3730244) | 6.477e-3 ± 3.61e-2 (n=949125) | 9.315e-3 ± 6.02e-2 (n=243342) | 4.053e-2 ± 1.68e-1 (n=89309) | 9.299e-2 ± 2.46e-1 (n=26575) | 2.856e-1 ± 4.07e-1 (n=8494) | 4.003e-1 ± 4.43e-1 (n=3946) | 4.119e-1 ± 4.45e-1 (n=1205) | 4.626e-1 ± 3.82e-1 (n=2650) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 1.143e-2 ± 4.01e-2 (n=4945110) | 9.927e-3 ± 3.70e-2 (n=3730244) | 7.472e-3 ± 3.61e-2 (n=949125) | 9.553e-3 ± 5.24e-2 (n=243342) | 3.239e-2 ± 1.25e-1 (n=89309) | 7.231e-2 ± 1.81e-1 (n=26575) | 2.166e-1 ± 2.98e-1 (n=8494) | 3.018e-1 ± 3.24e-1 (n=3946) | 3.115e-1 ± 3.29e-1 (n=1205) | 4.515e-1 ± 3.49e-1 (n=2650) |
| ModifiedCosine (mass-spectrometry-traits) | 6.260e-2 ± 1.75e-1 (n=4945110) | 5.260e-2 ± 1.66e-1 (n=3730244) | 4.013e-2 ± 1.50e-1 (n=949125) | 3.736e-2 ± 1.51e-1 (n=243342) | 6.877e-2 ± 2.25e-1 (n=89309) | 1.322e-1 ± 3.05e-1 (n=26575) | 3.338e-1 ± 4.50e-1 (n=8494) | 4.549e-1 ± 4.80e-1 (n=3946) | 4.815e-1 ± 4.82e-1 (n=1205) | 4.792e-1 ± 4.10e-1 (n=2650) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 6.163e-2 ± 1.41e-1 (n=4945110) | 5.043e-2 ± 1.31e-1 (n=3730244) | 3.760e-2 ± 1.18e-1 (n=949125) | 3.413e-2 ± 1.21e-1 (n=243342) | 6.177e-2 ± 1.92e-1 (n=89309) | 1.181e-1 ± 2.64e-1 (n=26575) | 3.108e-1 ± 4.09e-1 (n=8494) | 4.255e-1 ± 4.41e-1 (n=3946) | 4.501e-1 ± 4.45e-1 (n=1205) | 4.769e-1 ± 3.75e-1 (n=2650) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 6.184e-2 ± 1.14e-1 (n=4945110) | 4.901e-2 ± 1.03e-1 (n=3730244) | 3.557e-2 ± 9.38e-2 (n=949125) | 3.129e-2 ± 9.67e-2 (n=243342) | 5.081e-2 ± 1.46e-1 (n=89309) | 9.410e-2 ± 1.99e-1 (n=26575) | 2.397e-1 ± 3.05e-1 (n=8494) | 3.256e-1 ± 3.28e-1 (n=3946) | 3.501e-1 ± 3.40e-1 (n=1205) | 4.641e-1 ± 3.41e-1 (n=2650) |

### rdkit

| Series | 0.0-0.1 | 0.1-0.2 | 0.2-0.3 | 0.3-0.4 | 0.4-0.5 | 0.5-0.6 | 0.6-0.7 | 0.7-0.8 | 0.8-0.9 | 0.9-1.0 |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| CosineHungarian (mass-spectrometry-traits) | 4.596e-3 ± 3.32e-2 (n=1450746) | 5.581e-3 ± 3.67e-2 (n=3447927) | 5.738e-3 ± 3.73e-2 (n=2795167) | 6.994e-3 ± 5.45e-2 (n=1438157) | 5.587e-3 ± 3.97e-2 (n=587753) | 7.953e-3 ± 5.36e-2 (n=182596) | 2.011e-2 ± 1.03e-1 (n=50545) | 1.094e-1 ± 2.71e-1 (n=19438) | 2.270e-1 ± 3.74e-1 (n=11212) | 3.817e-1 ± 4.58e-1 (n=16459) |
| EntropySimilarityUnweighted (mass-spectrometry-traits) | 6.830e-3 ± 3.20e-2 (n=1450746) | 8.559e-3 ± 3.62e-2 (n=3447927) | 8.964e-3 ± 3.68e-2 (n=2795167) | 9.804e-3 ± 5.13e-2 (n=1438157) | 8.441e-3 ± 3.87e-2 (n=587753) | 1.086e-2 ± 4.95e-2 (n=182596) | 2.300e-2 ± 9.01e-2 (n=50545) | 9.980e-2 ± 2.25e-1 (n=19438) | 2.069e-1 ± 3.21e-1 (n=11212) | 3.643e-1 ± 4.20e-1 (n=16459) |
| EntropySimilarityWeighted (mass-spectrometry-traits) | 8.643e-3 ± 3.45e-2 (n=1450746) | 1.049e-2 ± 3.85e-2 (n=3447927) | 1.081e-2 ± 3.86e-2 (n=2795167) | 1.099e-2 ± 4.56e-2 (n=1438157) | 9.925e-3 ± 3.94e-2 (n=587753) | 1.237e-2 ± 4.85e-2 (n=182596) | 2.414e-2 ± 8.06e-2 (n=50545) | 8.379e-2 ± 1.70e-1 (n=19438) | 1.651e-1 ± 2.36e-1 (n=11212) | 2.955e-1 ± 3.26e-1 (n=16459) |
| ModifiedCosine (mass-spectrometry-traits) | 5.995e-2 ± 1.69e-1 (n=1450746) | 5.112e-2 ± 1.60e-1 (n=3447927) | 5.561e-2 ± 1.70e-1 (n=2795167) | 6.030e-2 ± 1.81e-1 (n=1438157) | 5.845e-2 ± 1.78e-1 (n=587753) | 6.488e-2 ± 1.88e-1 (n=182596) | 8.038e-2 ± 2.11e-1 (n=50545) | 1.734e-1 ± 3.19e-1 (n=19438) | 2.865e-1 ± 3.91e-1 (n=11212) | 4.159e-1 ± 4.57e-1 (n=16459) |
| ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 5.885e-2 ± 1.36e-1 (n=1450746) | 5.073e-2 ± 1.28e-1 (n=3447927) | 5.369e-2 ± 1.35e-1 (n=2795167) | 5.704e-2 ± 1.44e-1 (n=1438157) | 5.479e-2 ± 1.40e-1 (n=587753) | 5.953e-2 ± 1.47e-1 (n=182596) | 7.402e-2 ± 1.69e-1 (n=50545) | 1.557e-1 ± 2.64e-1 (n=19438) | 2.595e-1 ± 3.34e-1 (n=11212) | 3.946e-1 ± 4.18e-1 (n=16459) |
| ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 6.058e-2 ± 1.13e-1 (n=1450746) | 5.090e-2 ± 1.04e-1 (n=3447927) | 5.230e-2 ± 1.07e-1 (n=2795167) | 5.425e-2 ± 1.12e-1 (n=1438157) | 5.190e-2 ± 1.09e-1 (n=587753) | 5.534e-2 ± 1.16e-1 (n=182596) | 6.856e-2 ± 1.37e-1 (n=50545) | 1.337e-1 ± 2.05e-1 (n=19438) | 2.140e-1 ± 2.54e-1 (n=11212) | 3.246e-1 ± 3.27e-1 (n=16459) |

## Correlation: Spectral Similarity vs Structural Similarity

| Fingerprint | Algorithm | Pearson r | Pearson p | Spearman rho | Spearman p | n_pairs |
| --- | --- | --- | --- | --- | --- | --- |
| atompair | CosineHungarian (mass-spectrometry-traits) | 0.1616 | 0.00e0 | 0.0867 | 0.00e0 | 10000000 |
| atompair | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1753 | 0.00e0 | 0.0819 | 0.00e0 | 10000000 |
| atompair | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.1625 | 0.00e0 | 0.0813 | 0.00e0 | 10000000 |
| atompair | ModifiedCosine (mass-spectrometry-traits) | 0.0436 | 0.00e0 | 0.0235 | 0.00e0 | 10000000 |
| atompair | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.0530 | 0.00e0 | 0.0263 | 0.00e0 | 10000000 |
| atompair | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.0516 | 0.00e0 | 0.0237 | 0.00e0 | 10000000 |
| ecfp | CosineHungarian (mass-spectrometry-traits) | 0.3185 | 0.00e0 | 0.1196 | 0.00e0 | 10000000 |
| ecfp | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.3306 | 0.00e0 | 0.1075 | 0.00e0 | 10000000 |
| ecfp | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.3018 | 0.00e0 | 0.1082 | 0.00e0 | 10000000 |
| ecfp | ModifiedCosine (mass-spectrometry-traits) | 0.1061 | 0.00e0 | 0.0793 | 0.00e0 | 10000000 |
| ecfp | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.1303 | 0.00e0 | 0.0820 | 0.00e0 | 10000000 |
| ecfp | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.1422 | 0.00e0 | 0.0830 | 0.00e0 | 10000000 |
| fcfp | CosineHungarian (mass-spectrometry-traits) | 0.2680 | 0.00e0 | 0.1059 | 0.00e0 | 10000000 |
| fcfp | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.2842 | 0.00e0 | 0.0967 | 0.00e0 | 10000000 |
| fcfp | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.2650 | 0.00e0 | 0.0973 | 0.00e0 | 10000000 |
| fcfp | ModifiedCosine (mass-spectrometry-traits) | 0.0876 | 0.00e0 | 0.0593 | 0.00e0 | 10000000 |
| fcfp | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.1109 | 0.00e0 | 0.0618 | 0.00e0 | 10000000 |
| fcfp | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.1238 | 0.00e0 | 0.0627 | 0.00e0 | 10000000 |
| maccs | CosineHungarian (mass-spectrometry-traits) | 0.1364 | 0.00e0 | 0.0552 | 0.00e0 | 10000000 |
| maccs | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1463 | 0.00e0 | 0.0556 | 0.00e0 | 10000000 |
| maccs | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.1352 | 0.00e0 | 0.0554 | 0.00e0 | 10000000 |
| maccs | ModifiedCosine (mass-spectrometry-traits) | 0.0573 | 0.00e0 | 0.0181 | 0.00e0 | 10000000 |
| maccs | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.0660 | 0.00e0 | 0.0288 | 0.00e0 | 10000000 |
| maccs | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.0661 | 0.00e0 | 0.0276 | 0.00e0 | 10000000 |
| map | CosineHungarian (mass-spectrometry-traits) | 0.1375 | 0.00e0 | -0.0309 | 0.00e0 | 10000000 |
| map | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1276 | 0.00e0 | -0.0232 | 0.00e0 | 10000000 |
| map | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.0946 | 0.00e0 | -0.0245 | 0.00e0 | 10000000 |
| map | ModifiedCosine (mass-spectrometry-traits) | -0.0084 | 0.00e0 | -0.1163 | 0.00e0 | 10000000 |
| map | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | -0.0184 | 0.00e0 | -0.1128 | 0.00e0 | 10000000 |
| map | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | -0.0440 | 0.00e0 | -0.1169 | 0.00e0 | 10000000 |
| rdkit | CosineHungarian (mass-spectrometry-traits) | 0.1292 | 0.00e0 | 0.0276 | 0.00e0 | 10000000 |
| rdkit | EntropySimilarityUnweighted (mass-spectrometry-traits) | 0.1305 | 0.00e0 | 0.0348 | 0.00e0 | 10000000 |
| rdkit | EntropySimilarityWeighted (mass-spectrometry-traits) | 0.1111 | 0.00e0 | 0.0340 | 0.00e0 | 10000000 |
| rdkit | ModifiedCosine (mass-spectrometry-traits) | 0.0446 | 0.00e0 | -0.0186 | 0.00e0 | 10000000 |
| rdkit | ModifiedLinearEntropyUnweighted (mass-spectrometry-traits) | 0.0445 | 0.00e0 | -0.0023 | 2.54e-13 | 10000000 |
| rdkit | ModifiedLinearEntropyWeighted (mass-spectrometry-traits) | 0.0319 | 0.00e0 | -0.0059 | 0.00e0 | 10000000 |

