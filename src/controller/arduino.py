from controller.config import ArduinoConfig
from controller.temperature import TemperatureReading


class ArduinoTempController:
    def __init__(self, config: ArduinoConfig):
        self.config = config

    def read_temperature(self) -> TemperatureReading:
        # TODO: implement
        return TemperatureReading()
