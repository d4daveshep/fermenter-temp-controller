# test_mock_serial_connection.py
import json
import math
from typing import Any

from mock_arduino_controller.mock_temperature import (
    mock_arduino_output,
    sine_wave_value,
    step_value,
)


def test_sine_wave():
    min_value: float = 19.0
    max_value: float = 21.0
    total: float = 0.0

    for interval in range(20):
        value = sine_wave_value(interval, min=min_value, max=max_value)
        total += value

        # check value is between min and max
        assert min_value <= value <= max_value

        # check values repeat each interval
        assert sine_wave_value(interval + 20, min=min_value, max=max_value) == value

    # check average value is mean of min and max
    assert math.isclose(total / 20.0, (min_value + max_value) / 2.0)


def test_step_value():
    low_value: float = 18.0
    mid_value: float = 20.0
    high_value: float = 22.0

    for interval in range(60):
        value = step_value(interval, low=low_value, mid=mid_value, high=high_value)
        if interval % 60 < 20:
            assert math.isclose(value, low_value), f"interval:{interval}"
        elif interval % 60 < 40:
            assert math.isclose(value, mid_value), f"interval:{interval}"
        elif interval % 60 < 60:
            assert math.isclose(value, high_value), f"interval:{interval}"


def test_mock_arduino_output():
    arduino_output: str = mock_arduino_output(interval=0, target=21.3)
    arduino_json: dict[str, Any] = json.loads(arduino_output)

    assert "instant" in arduino_json
    assert arduino_json["instant"] == "0"

    assert "average" in arduino_json
    assert math.isclose(float(arduino_json["average"]), sine_wave_value(interval=0))

    assert "min" in arduino_json
    assert math.isclose(float(arduino_json["min"]), sine_wave_value(interval=0))

    assert "max" in arduino_json
    assert math.isclose(float(arduino_json["max"]), sine_wave_value(interval=0))

    assert "target" in arduino_json
    assert math.isclose(float(arduino_json["target"]), 21.3)

    assert "ambient" in arduino_json
    assert math.isclose(float(arduino_json["ambient"]), step_value(interval=0))

    assert "action" in arduino_json
    assert arduino_json["action"] == "Heat"

    assert "heat" in arduino_json
    assert arduino_json["heat"]

    assert "reason-code" in arduino_json
    assert arduino_json["reason-code"].startswith("RC")

    assert "json-size" in arduino_json
    assert arduino_json["json-size"] == 12345
