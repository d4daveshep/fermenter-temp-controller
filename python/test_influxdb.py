import pytest
import requests as requests

from config import ControllerConfig
from temperature_database import TemperatureDatabase


def test_influxdb_server_is_available():
    config = ControllerConfig("./test_valid_config_file.txt")
    temperature_database = TemperatureDatabase(config)
    assert temperature_database.is_server_available()


