from __future__ import annotations

from matchms.similarity import ModifiedCosine

from python_ref.types import SpectrumData


def compute_once(
    left: SpectrumData, right: SpectrumData, params: dict[str, object]
) -> tuple[float, int]:
    scorer = ModifiedCosine(
        tolerance=params["tolerance"],
        mz_power=params["mz_power"],
        intensity_power=params["intensity_power"],
    )
    result = scorer.pair(left.spectrum, right.spectrum)
    return float(result["score"]), int(result["matches"])
