from __future__ import annotations

from python_ref.algorithms.ms_entropy_api import entropy_similarity_ms_entropy
from python_ref.types import SpectrumData


def compute_once(
    left: SpectrumData, right: SpectrumData, params: dict[str, object]
) -> tuple[float, int]:
    tolerance = float(params["tolerance"])
    score = entropy_similarity_ms_entropy(left, right, tolerance, weighted=False)
    if score is None:
        raise RuntimeError(
            "Unsupported ms_entropy API for entropy similarity. "
            "Please install a compatible ms_entropy version."
        )
    return score, -1
