from datetime import datetime

import pytest
from influxdb_client import Point, WritePrecision
from influxdb_client.rest import ApiException

from controller.config import ControllerConfig
from controller.temp_controller import TempController
from controller.temperature_database import TemperatureDatabase


def test_database_server_is_available():
    config = ControllerConfig("test_valid_config_file.ini")
    temperature_database = TemperatureDatabase(config)
    assert temperature_database.is_server_available()


def test_database_server_is_unavailable():
    config = ControllerConfig("test_valid_config_file.ini")
    config.influxdb_url = "http://localhost:9999"

    # with pytest.raises(requests.exceptions.ConnectionError) as err_info:
    temperature_database = TemperatureDatabase(config)
    assert not temperature_database.is_server_available()


def test_open_database_connection():
    config = ControllerConfig("test_valid_config_file.ini")
    temperature_database = TemperatureDatabase(config)
    assert temperature_database.is_server_available()

    database_client = temperature_database.get_database_client()
    assert database_client.ping()
    assert database_client.ready()
    # database_client.close()


@pytest.fixture
def temperature_database():
    config = ControllerConfig("test_valid_config_file.ini")
    my_temperature_database = TemperatureDatabase(config)
    assert my_temperature_database.is_server_available()
    return my_temperature_database


def test_create_point_object(temperature_database):
    time_stamp = datetime.utcnow()
    point_1 = temperature_database.create_point(
        fermenter_temp=21.3, ambient_temp=15.6, target_temp=20.0, timestamp=time_stamp)

    point_2 = Point("temperature") \
        .tag("brew-id", "00-test-v00") \
        .field("fermenter", 21.3) \
        .field("ambient", 15.6) \
        .field("target", 20.0) \
        .time(datetime.utcnow(), WritePrecision.MS)

    assert point_1.__eq__(point_2)




def test_write_record_to_database_from_fermenter_json(temperature_database):
    timestamp = datetime.utcnow()

    json_dict = {"instant": 8.0625, "average": 8.178817, "min": 8.0625, "max": 12.4375, "target": 19,
                 "ambient": 19.64673, "action": "Heat", "heat": True, "reason-code": "RC1",
                 "timestamp": timestamp.timestamp(),
                 "json-size": 89}

    json_dict = TempController.convert_dict_int_values_to_float(json_dict)

    point = temperature_database.create_point_from_fermenter_json_dict(json_dict)

    temperature_database.write_temperature_record(point)

    results_dict = temperature_database.get_last_record()
    assert results_dict["instant"] == 8.0625
    assert results_dict["average"] == 8.178817
    assert results_dict["min"] == 8.0625
    assert results_dict["max"] == 12.4375
    assert results_dict["target"] == 19.0
    assert results_dict["ambient"] == 19.64673
    assert results_dict["action"] == "Heat"
    assert results_dict["heat"]
    assert results_dict["reason-code"] == "RC1"
    assert results_dict["json-size"] == 89

    assert results_dict["time"].hour == timestamp.hour
    assert results_dict["time"].minute == timestamp.minute
    assert results_dict["time"].second == timestamp.second


def test_write_record_to_database_with_wrong_org(temperature_database):
    config = temperature_database.config
    config.influxdb_org = "wrong.org"
    point = temperature_database.create_point(fermenter_temp=21.3, ambient_temp=15.6, target_temp=20.0,
                                              timestamp=datetime.utcnow())

    with pytest.raises(ApiException) as err_info:
        temperature_database.write_temperature_record(point)

    assert err_info.value.args[0] == f"organization name \"{config.influxdb_org}\" not found"


def test_write_record_to_database_with_wrong_bucket(temperature_database):
    config = temperature_database.config
    config.influxdb_bucket = "wrong_bucket"
    point = temperature_database.create_point(fermenter_temp=21.3, ambient_temp=15.6, target_temp=20.0,
                                              timestamp=datetime.utcnow())

    with pytest.raises(ApiException) as err_info:
        temperature_database.write_temperature_record(point)

    assert err_info.value.args[0] == f"bucket \"{config.influxdb_bucket}\" not found"


def test_write_record_to_database_with_wrong_token(temperature_database):
    config = temperature_database.config
    config.influxdb_token = "wrong-token"
    point = temperature_database.create_point(fermenter_temp=21.3, ambient_temp=15.6, target_temp=20.0,
                                              timestamp=datetime.utcnow())

    with pytest.raises(ApiException) as err_info:
        temperature_database.write_temperature_record(point)

    assert err_info.value.args[0] == "unauthorized access"
