import math
import json
from typing import Any


def sine_wave_value(interval: int, min: float = 19.0, max: float = 21.0) -> float:
    """
    Return floats between min and max using a sine function at the specified interval.
    Full cycle is split into 20 intervals.
    """
    intervals: int = 20
    avg: float = (min + max) / 2.0
    angle: float = (2 * math.pi * (interval % intervals)) / intervals
    sine_value: float = math.sin(angle)
    return avg + sine_value


def step_value(
    interval: int, low: float = 18.0, mid: float = 20.0, high: float = 22.0
) -> float:
    """
    Return floats between low, mid and high using a step function at the specified interval.
    """
    intervals: int = 60

    if interval % intervals < 20:
        return low
    elif interval % intervals < 40:
        return mid
    elif interval % intervals < 60:
        return high
    else:
        raise ValueError(f"can't calculate step value for interval={interval}")


def mock_arduino_output(interval: int) -> str:
    arduino_dict: dict[str, Any] = {
        "instant": str(interval),
        "average": str(sine_wave_value(interval)),
    }
    arduino_str: str = json.dumps(arduino_dict)
    return arduino_str
