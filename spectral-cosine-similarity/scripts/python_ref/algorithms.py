"""Algorithm-specific compute adapters (consolidated)."""

from __future__ import annotations

import numpy as np
import ms_entropy  # type: ignore
from matchms.similarity import CosineGreedy, CosineHungarian, ModifiedCosine

from python_ref.types import SpectrumData


def _matchms_compute(scorer_cls, left: SpectrumData, right: SpectrumData, params: dict) -> tuple[float, int]:
    scorer = scorer_cls(
        tolerance=params["tolerance"],
        mz_power=params["mz_power"],
        intensity_power=params["intensity_power"],
    )
    result = scorer.pair(left.spectrum, right.spectrum)
    return float(result["score"]), int(result["matches"])


def cosine_hungarian(left: SpectrumData, right: SpectrumData, params: dict) -> tuple[float, int]:
    return _matchms_compute(CosineHungarian, left, right, params)


def cosine_greedy(left: SpectrumData, right: SpectrumData, params: dict) -> tuple[float, int]:
    return _matchms_compute(CosineGreedy, left, right, params)


def modified_greedy_cosine(left: SpectrumData, right: SpectrumData, params: dict) -> tuple[float, int]:
    return _matchms_compute(ModifiedCosine, left, right, params)


def _clean_peaks(data: SpectrumData) -> np.ndarray:
    peaks = np.column_stack((data.mz, data.intensities)).astype(np.float32)
    peaks = np.ascontiguousarray(peaks)
    return ms_entropy.clean_spectrum(peaks)


def entropy_weighted(left: SpectrumData, right: SpectrumData, params: dict) -> tuple[float, int]:
    tolerance = float(params["tolerance"])
    score = float(ms_entropy.calculate_entropy_similarity(
        peaks_a=_clean_peaks(left),
        peaks_b=_clean_peaks(right),
        ms2_tolerance_in_da=tolerance,
        clean_spectra=False,
    ))
    return score, -1


def entropy_unweighted(left: SpectrumData, right: SpectrumData, params: dict) -> tuple[float, int]:
    tolerance = float(params["tolerance"])
    score = float(ms_entropy.calculate_unweighted_entropy_similarity(
        peaks_a=_clean_peaks(left),
        peaks_b=_clean_peaks(right),
        ms2_tolerance_in_da=tolerance,
        clean_spectra=False,
    ))
    return score, -1


ALGORITHMS: list[tuple[str, str, object]] = [
    ("CosineHungarian", "matchms", cosine_hungarian),
    ("CosineGreedy", "matchms", cosine_greedy),
    ("ModifiedGreedyCosine", "matchms", modified_greedy_cosine),
    ("EntropySimilarityWeighted", "ms_entropy", entropy_weighted),
    ("EntropySimilarityUnweighted", "ms_entropy", entropy_unweighted),
]
