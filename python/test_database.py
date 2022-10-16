from datetime import datetime

import pytest
from influxdb_client import Point, WritePrecision
from influxdb_client.rest import ApiException

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
    assert database_client.ping()
    assert database_client.ready()
    # database_client.close()


@pytest.fixture
def temperature_database():
    config = ControllerConfig("./test_valid_config_file.txt")
    my_temperature_database = TemperatureDatabase(config)
    assert my_temperature_database.is_server_available()
    return my_temperature_database


def test_create_point_object(temperature_database):
    time_stamp = datetime.utcnow()
    point_1 = temperature_database.create_point(
        fermenter_temp=21.3, ambient_temp=15.6, timestamp=time_stamp)

    point_2 = Point("temperature") \
        .tag("brew-id", "00-test-v00") \
        .field("fermenter", 21.3) \
        .field("ambient", 15.6) \
        .time(datetime.utcnow(), WritePrecision.MS)

    assert point_1.__eq__(point_2)


def test_write_record_to_database(temperature_database):
    point = temperature_database.create_point(
        fermenter_temp=21.3, ambient_temp=15.6, timestamp=datetime.utcnow())

    temperature_database.write_temperature_record(point)

    # verify record
    query_string = 'from(bucket: "temp-test") |> range(start: -1m)'
    client = temperature_database.get_database_client()
    query_api = client.query_api()

    tables = query_api.query(query_string)
    ambient = tables[0].records[0].values["_value"]
    fermenter = tables[1].records[0].values["_value"]

    assert ambient == 15.6
    assert fermenter == 21.3


def test_write_record_to_database_with_wrong_org(temperature_database):
    config = temperature_database.get_config()
    config.influxdb_org = "wrong.org"
    point = temperature_database.create_point(
        fermenter_temp=21.3, ambient_temp=15.6, timestamp=datetime.utcnow())

    with pytest.raises(ApiException) as err_info:
        temperature_database.write_temperature_record(point)

    assert err_info.value.args[0] == f"organization name \"{config.influxdb_org}\" not found"

def test_write_record_to_database_with_wrong_bucket(temperature_database):
    config = temperature_database.get_config()
    config.influxdb_bucket = "wrong_bucket"
    point = temperature_database.create_point(
        fermenter_temp=21.3, ambient_temp=15.6, timestamp=datetime.utcnow())

    with pytest.raises(ApiException) as err_info:
        temperature_database.write_temperature_record(point)

    assert err_info.value.args[0] == f"bucket \"{config.influxdb_bucket}\" not found"

def test_write_record_to_database_with_wrong_token(temperature_database):
    config = temperature_database.get_config()
    config.influxdb_token = "wrong-token"
    point = temperature_database.create_point(
        fermenter_temp=21.3, ambient_temp=15.6, timestamp=datetime.utcnow())

    with pytest.raises(ApiException) as err_info:
        temperature_database.write_temperature_record(point)

    assert err_info.value.args[0] == "unauthorized access"
