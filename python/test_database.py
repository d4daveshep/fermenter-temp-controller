import pytest
import requests as requests

from config import ControllerConfig
from temperature_database import TemperatureDatabase


def test_database_server_is_available():
    config = ControllerConfig("./test_valid_config_file.txt")
    temperature_database = TemperatureDatabase(config)
    assert temperature_database.is_server_available()

def test_database_server_is_unavailable():
    config = ControllerConfig("./test_valid_config_file.txt")
    config.influxdb_url = "http://localhost:9999"

    # with pytest.raises(requests.exceptions.ConnectionError) as err_info:
    temperature_database = TemperatureDatabase(config)
    assert not temperature_database.is_server_available()



def test_open_database_connection():
    config = ControllerConfig("./test_valid_config_file.txt")
    temperature_database = TemperatureDatabase(config)
    assert temperature_database.is_server_available()

    database_client = temperature_database.get_database_client()
    assert database_client