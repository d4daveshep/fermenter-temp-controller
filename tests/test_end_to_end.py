from controller.config import ControllerConfig
from controller.arduino import ArduinoTempController
from controller.temperature import TemperatureReading
from pathlib import Path


def test_end_to_end_sync():
    # read the configuration file
    config: ControllerConfig = ControllerConfig.load_config(
        filename=Path("tests/valid_config.ini")
    )

    # get temperature readings from Arduino temperature controller
    arduino_temp_controller: ArduinoTempController = ArduinoTempController(
        config.arduino
    )
    temperature_reading: TemperatureReading = arduino_temp_controller.read_temperature()
    assert temperature_reading.target_temp == 20.0

    # put temperature readings in database
    # read latest readings from database
    # update the target temperature
    # get temperature readings from sensor
    # put temperature readings in database
    # read latest readings from database
    # update the brew_id
    # get temperature readings from sensor
    # put temperature readings in database
    # read latest readings from database
    assert False
