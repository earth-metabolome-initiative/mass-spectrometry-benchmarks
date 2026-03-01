from __future__ import annotations

import numpy as np

import ms_entropy  # type: ignore

from python_ref.types import SpectrumData


def _to_peaks(data: SpectrumData) -> np.ndarray:
    """Build an (n, 2) float32 C-contiguous peak array with intensities
    normalized to sum to 1.

    ms_entropy 1.4.0 Cython uses ``np.array(x, copy=False)`` when
    ``clean_spectra=False``, which raises on NumPy >= 2.0 if a dtype
    conversion (float64 -> float32) would be needed.  Pre-converting to
    float32 avoids this.

    Normalizing to sum-to-one is required because the Cython/C
    ``calculate_unweighted_entropy_similarity`` does NOT normalize
    internally when ``clean_spectra=False`` — it feeds raw intensities
    straight into the JSD formula.  The
    ``calculate_entropy_similarity`` (weighted) path normalizes inside
    its ``apply_weight_to_intensity`` C helper so pre-normalizing is
    harmless there (the power-then-renormalize step is scale-invariant).
    """
    peaks = np.column_stack((data.mz, data.intensities)).astype(np.float32)
    intensity_sum = peaks[:, 1].sum()
    if intensity_sum > 0:
        peaks[:, 1] /= intensity_sum
    return np.ascontiguousarray(peaks)


def entropy_similarity_ms_entropy(
    left: SpectrumData, right: SpectrumData, tolerance: float, weighted: bool
) -> float | None:
    peaks_a = _to_peaks(left)
    peaks_b = _to_peaks(right)

    def try_call(func, **kwargs):
        try:
            return float(func(**kwargs))
        except TypeError:
            return None
        except Exception:
            return None

    if weighted:
        func = getattr(ms_entropy, "calculate_entropy_similarity", None)
        if func is None:
            return None
        # ms_entropy 1.4.0 Cython uses positional param names peaks_a/peaks_b.
        # Older pure-Python versions used spectrum_a/spectrum_b or spectra_a/spectra_b.
        for kwargs in (
            {
                "peaks_a": peaks_a,
                "peaks_b": peaks_b,
                "ms2_tolerance_in_da": tolerance,
                "clean_spectra": False,
            },
            {
                "spectrum_a": peaks_a,
                "spectrum_b": peaks_b,
                "ms2_tolerance_in_da": tolerance,
                "clean_spectra": False,
            },
            {
                "spectrum_a": peaks_a,
                "spectrum_b": peaks_b,
                "ms2_tolerance": tolerance,
                "clean_spectra": False,
            },
            {
                "spectra_a": peaks_a,
                "spectra_b": peaks_b,
                "ms2_tolerance_in_da": tolerance,
                "clean_spectra": False,
            },
        ):
            result = try_call(func, **kwargs)
            if result is not None:
                return result
    else:
        func = getattr(ms_entropy, "calculate_unweighted_entropy_similarity", None)
        if func is not None:
            for kwargs in (
                {
                    "peaks_a": peaks_a,
                    "peaks_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                },
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                },
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance": tolerance,
                    "clean_spectra": False,
                },
            ):
                result = try_call(func, **kwargs)
                if result is not None:
                    return result

        func = getattr(ms_entropy, "calculate_entropy_similarity", None)
        if func is not None:
            for kwargs in (
                {
                    "peaks_a": peaks_a,
                    "peaks_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                    "weighted": False,
                },
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                    "weighted": False,
                },
                {
                    "spectrum_a": peaks_a,
                    "spectrum_b": peaks_b,
                    "ms2_tolerance_in_da": tolerance,
                    "clean_spectra": False,
                    "use_weighted_intensity": False,
                },
            ):
                result = try_call(func, **kwargs)
                if result is not None:
                    return result

    return None
