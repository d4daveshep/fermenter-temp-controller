# test_mock_serial_connection.py
from mock_arduino_controller.mock_temperature import sine_wave_value


def test_sine_wave():
    min_value: float = 19.0
    max_value: float = 21.0

    for interval in range(20):
        value = sine_wave_value(interval, min=min_value, max=max_value)
        assert min_value <= value <= max_value
