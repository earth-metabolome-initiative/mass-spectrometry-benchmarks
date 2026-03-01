from __future__ import annotations

from dataclasses import dataclass
from typing import Any, Callable

import numpy as np
from matchms import Spectrum


@dataclass(frozen=True)
class ExperimentData:
    id: int
    params: dict[str, Any]


@dataclass
class SpectrumData:
    spectrum: Spectrum
    mz: np.ndarray
    intensities: np.ndarray


@dataclass(frozen=True)
class WorkItem:
    left_id: int
    right_id: int
    experiment: ExperimentData


ComputeFn = Callable[[SpectrumData, SpectrumData, dict[str, Any]], tuple[float, int]]
