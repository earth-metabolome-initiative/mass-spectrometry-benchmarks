from __future__ import annotations

import numpy as np

import ms_entropy  # type: ignore

from python_ref.types import SpectrumData


def _clean_peaks(data: SpectrumData) -> np.ndarray:
    """Build a cleaned (n, 2) float32 peak array via ``ms_entropy.clean_spectrum``.

    This mirrors the Rust ``MsEntropyCleanSpectrum`` preprocessing: centroiding,
    noise filtering, and intensity normalization are all performed by the library's
    own ``clean_spectrum()`` with default parameters (``min_ms2_difference_in_da=0.05``,
    ``noise_threshold=0.01``, ``normalize_intensity=True``).

    The similarity functions are then called with ``clean_spectra=False`` because
    the spectra are already clean.
    """
    peaks = np.column_stack((data.mz, data.intensities)).astype(np.float32)
    peaks = np.ascontiguousarray(peaks)
    return ms_entropy.clean_spectrum(peaks)


def entropy_similarity_ms_entropy(
    left: SpectrumData, right: SpectrumData, tolerance: float, weighted: bool
) -> float | None:
    peaks_a = _clean_peaks(left)
    peaks_b = _clean_peaks(right)

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
