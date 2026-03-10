"""Algorithm-specific compute adapters (consolidated)."""

from __future__ import annotations

from collections.abc import Callable
from typing import Any, TypedDict, cast

import ms_entropy
import numpy as np
from matchms.similarity import (
    CosineGreedy,
    CosineHungarian,
    ModifiedCosineGreedy,
    ModifiedCosineHungarian,
)
from numpy.typing import NDArray

from python_ref.types import ComputeFn, SpectrumData


class MatchmsPairResult(TypedDict):
    score: float
    matches: int


def _matchms_compute(
    scorer_cls: Callable[..., Any],
    left: SpectrumData,
    right: SpectrumData,
    params: dict[str, Any],
) -> tuple[float, int]:
    scorer = scorer_cls(
        tolerance=params["tolerance"],
        mz_power=params["mz_power"],
        intensity_power=params["intensity_power"],
    )
    result = cast(MatchmsPairResult, scorer.pair(left.spectrum, right.spectrum))
    return float(result["score"]), int(result["matches"])


def cosine_hungarian(
    left: SpectrumData, right: SpectrumData, params: dict[str, Any]
) -> tuple[float, int]:
    return _matchms_compute(CosineHungarian, left, right, params)


def cosine_greedy(
    left: SpectrumData, right: SpectrumData, params: dict[str, Any]
) -> tuple[float, int]:
    return _matchms_compute(CosineGreedy, left, right, params)


def modified_greedy_cosine(
    left: SpectrumData, right: SpectrumData, params: dict[str, Any]
) -> tuple[float, int]:
    return _matchms_compute(ModifiedCosineGreedy, left, right, params)


def modified_cosine_hungarian(
    left: SpectrumData, right: SpectrumData, params: dict[str, Any]
) -> tuple[float, int]:
    return _matchms_compute(ModifiedCosineHungarian, left, right, params)


def _clean_peaks(data: SpectrumData) -> NDArray[np.float32]:
    peaks = np.column_stack((data.mz, data.intensities)).astype(np.float32, copy=False)
    peaks = np.ascontiguousarray(peaks)
    cleaned = ms_entropy.clean_spectrum(peaks)
    return np.asarray(cleaned, dtype=np.float32)


def entropy_weighted(
    left: SpectrumData, right: SpectrumData, params: dict[str, Any]
) -> tuple[float, int]:
    tolerance = float(params["tolerance"])
    score = float(
        ms_entropy.calculate_entropy_similarity(
            peaks_a=_clean_peaks(left),
            peaks_b=_clean_peaks(right),
            ms2_tolerance_in_da=tolerance,
            clean_spectra=False,
        )
    )
    return score, -1


def entropy_unweighted(
    left: SpectrumData, right: SpectrumData, params: dict[str, Any]
) -> tuple[float, int]:
    tolerance = float(params["tolerance"])
    score = float(
        ms_entropy.calculate_unweighted_entropy_similarity(
            peaks_a=_clean_peaks(left),
            peaks_b=_clean_peaks(right),
            ms2_tolerance_in_da=tolerance,
            clean_spectra=False,
        )
    )
    return score, -1


ALGORITHMS: list[tuple[str, str, ComputeFn]] = [
    ("CosineHungarian", "matchms", cosine_hungarian),
    ("CosineGreedy", "matchms", cosine_greedy),
    ("ModifiedGreedyCosine", "matchms", modified_greedy_cosine),
    ("ModifiedCosineHungarian", "matchms", modified_cosine_hungarian),
    ("EntropySimilarityWeighted", "ms_entropy", entropy_weighted),
    ("EntropySimilarityUnweighted", "ms_entropy", entropy_unweighted),
]
