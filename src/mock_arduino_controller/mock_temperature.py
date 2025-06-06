import math


def sine_wave_value(interval: int) -> float:
    """
    Return floats between 19.0 and 21.0 using a sine function.
    Full cycle is split into 20 intervals.
    """
    intervals: int = 20
    while True:
        angle: float = (2 * math.pi * (interval % intervals)) / intervals
        sine_value: float = math.sin(angle)
        scaled_value: float = 20.0 + sine_value
