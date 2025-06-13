import json
import math
from typing import Any, Generator

from mock_arduino_controller.mock_temperature import (
    mock_arduino_output,
    mock_arduino_output_generator,
    sine_wave_value,
    step_value,
)


def test_sine_wave():
    """
    Test the sine wave function we will use for generating fluctuating fermenter temperature values
    """
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
    """
    Test the step value function we will use for generating fluctuating ambient temperature values
    """
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


def test_mock_arduino_temperature_reading_output():
    """
    Test the json string output from our mock temperature reading function
    """
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


def test_mock_auduino_serial_output_generator_with_target_temp_function():
    """
    Test the target temp function in the mock_arduino_output_generator
    """
    # create some data
    data: list[float] = [11.1]

    # define a function to get the latest data item
    def get_latest_data() -> float:
        return data[-1]

    # create a generator using the latest data function
    output_generator: Generator = mock_arduino_output_generator(
        start_interval=0, get_target=get_latest_data
    )

    # get next generator output and check the target temp value
    next_output: bytes = next(output_generator)
    json_output: dict[str, Any] = json.loads(next_output.decode())
    assert json_output["target"] == "11.1"

    # add another item to the data
    data.append(22.2)

    # get next generator output and check temp value is latest
    next_output: bytes = next(output_generator)
    json_output: dict[str, Any] = json.loads(next_output.decode())
    assert json_output["target"] == "22.2"


def test_mock_auduino_serial_output_generator_with_no_target_temp_function():
    """
    Test the mock_arduino_output_generator without specifying a target temp function
    """
    # create a generator without a get target function
    output_generator: Generator = mock_arduino_output_generator(start_interval=0)

    # get next generator output and check the target temp value
    next_output: bytes = next(output_generator)
    json_output: dict[str, Any] = json.loads(next_output.decode())
    assert json_output["target"] == "20.0"

    # get next generator output and check temp value is latest
    next_output: bytes = next(output_generator)
    json_output: dict[str, Any] = json.loads(next_output.decode())
    assert json_output["target"] == "20.0"
