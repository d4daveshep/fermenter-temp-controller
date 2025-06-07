import math


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
