from __future__ import annotations

from collections.abc import Callable
from dataclasses import dataclass
from typing import Any

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


ComputeFn = Callable[[SpectrumData, SpectrumData, dict[str, Any]], tuple[float, int]]
