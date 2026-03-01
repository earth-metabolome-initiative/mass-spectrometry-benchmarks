from __future__ import annotations

import numpy as np

import ms_entropy  # type: ignore

from python_ref.types import SpectrumData


def entropy_similarity_ms_entropy(
    left: SpectrumData, right: SpectrumData, tolerance: float, weighted: bool
) -> float | None:
    peaks_a = np.column_stack((left.mz, left.intensities))
    peaks_b = np.column_stack((right.mz, right.intensities))

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
        for kwargs in (
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
        for args in ((peaks_a, peaks_b), (peaks_a, peaks_b, tolerance)):
            try:
                return float(func(*args))
            except Exception:
                pass
    else:
        func = getattr(ms_entropy, "calculate_unweighted_entropy_similarity", None)
        if func is not None:
            for kwargs in (
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
            for args in ((peaks_a, peaks_b), (peaks_a, peaks_b, tolerance)):
                try:
                    return float(func(*args))
                except Exception:
                    pass

        func = getattr(ms_entropy, "calculate_entropy_similarity", None)
        if func is not None:
            for kwargs in (
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
