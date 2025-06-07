# test_mock_serial_connection.py
from mock_arduino_controller.mock_temperature import sine_wave_value, step_value


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
    assert total / 20.0 - (min_value + max_value) / 2.0 < 0.0001


def test_step_value():
    low_value: float = 18.0
    mid_value: float = 20.0
    high_value: float = 22.0
    tolerance: float = 0.0001

    for interval in range(60):
        value = step_value(interval, low=low_value, mid=mid_value, high=high_value)
        if interval % 60 < 20:
            assert value - low_value < tolerance, f"interval:{interval}"
        elif interval % 60 < 40:
            assert value - mid_value < tolerance, f"interval:{interval}"
        elif interval % 60 < 60:
            assert value - high_value < tolerance, f"interval:{interval}"


def test_arduino_output_generator():
    assert False, "write this test"
