# test_mock_serial_connection.py
from mock_arduino_controller.mock_temperature import sine_wave_value


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
