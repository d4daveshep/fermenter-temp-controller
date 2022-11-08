from datetime import datetime

import pytest
from fastapi.testclient import TestClient

from controller.config import ControllerConfig
from controller.temperature_database import TemperatureDatabase
from web.fastapi_app import app

client = TestClient(app)

@pytest.fixture
def temperature_database():
    config = ControllerConfig("test_valid_config_file.ini")
    my_temperature_database = TemperatureDatabase(config)
    assert my_temperature_database.is_server_available()
    return my_temperature_database

def test_read_main(temperature_database):

    timestamp = datetime.utcnow()
    point = temperature_database.create_point(fermenter_temp=21.3, ambient_temp=15.6, target_temp=20.0,
                                              timestamp=timestamp)
    temperature_database.write_temperature_record(point)

    response = client.get("/")
    assert response.status_code == 200
    assert response.json()["fermenter"] == 21.3
    assert response.json()["ambient"] == 15.6
    assert response.json()["target"] == 20.0
