import pytest
import requests as requests
from datetime import datetime

from influxdb_client import Point, WritePrecision

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

def test_write_record_to_database():
    config = ControllerConfig("./test_valid_config_file.txt")
    temperature_database = TemperatureDatabase(config)
    assert temperature_database.is_server_available()

    point = Point("temperature") \
        .tag("brew-id", "00-test-v00") \
        .field("fermenter", 21.3) \
        .field("ambient", 15.6) \
        .time(datetime.utcnow(), WritePrecision.MS)

    temperature_database.write_temperature_record(point)

    # verify record
    query_string = 'from(bucket: "temp-test") |> range(start: -1m)'
  #           "|> filter(fn: (r) => r["_measurement"] == "temperature")" \
  #           "|> filter(fn: (r) => r["_field"] == "ambient" or r["_field"] == "fermenter")" \
  #           "|> filter(fn: (r) => r["brew-id"] == "00-test-v00")
  # |> aggregateWindow(every: v.windowPeriod, fn: mean, createEmpty: false)
  # |> yield(name: "mean")"
    client = temperature_database.get_database_client()
    query_api = client.query_api()

    tables = query_api.query(query_string)
    ambient = tables[0].records[0].values["_value"]
    fermenter = tables[1].records[0].values["_value"]

    assert ambient == 15.6
    assert fermenter == 21.3


def test_open_database_connection_with_invalid_creditials():

    config = ControllerConfig("./test_valid_config_file.txt")
    temperature_database = TemperatureDatabase(config)
    assert temperature_database.is_server_available()

    config.influxdb_org = "bad.org"
    # config.influxdb_token = "random-token"
    # config.influxdb_bucket = "no_bucket"

    # with pytest.raises(Exception) as err_info:
    database_client = temperature_database.get_database_client()
    ping = database_client.ping()
    ready = database_client.ready()

    assert False