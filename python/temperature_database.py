import json
from datetime import datetime

import requests
from influxdb_client import InfluxDBClient, Point, WritePrecision
from influxdb_client.client.write_api import SYNCHRONOUS

from config import ControllerConfig


class TemperatureDatabase:
    def __init__(self, config: ControllerConfig):
        self.__database_client = None
        self.__config = config

    def get_config(self):
        return self.__config

    def create_point(self, fermenter_temp: float, ambient_temp: float, target_temp: float,
                     timestamp: datetime) -> Point:
        point = Point("temperature") \
            .tag("brew-id", self.__config.brew_id) \
            .field("fermenter", fermenter_temp) \
            .field("ambient", ambient_temp) \
            .field("target", target_temp) \
            .time(timestamp, WritePrecision.MS)
        return point

    def create_point_from_fermenter_json(self, json_dict: dict) -> Point:

        # json_dict = json.loads(json_string)
        timestamp = datetime.utcnow()

        point = Point("temperature").tag("brew-id", self.__config.brew_id)

        for key, value in json_dict.items():
            point.field(key, value)

        point.time(timestamp, WritePrecision.MS)
        return point

    def is_server_available(self):
        # run equivalant of curl -sL -I localhost:8086/ping?wait_for_leader=30s
        url = self.__config.influxdb_url + "/ping"
        params = {'wait_for_leader': "30s"}

        try:
            response = requests.get(url=url, params=params)
            return response.status_code == 204 and \
                   response.headers["X-Influxdb-Build"] == "OSS" and \
                   response.headers["X-Influxdb-Version"] >= "v2.4"
        except requests.exceptions.ConnectionError as exec:
            return False

    def get_database_client(self) -> InfluxDBClient:
        if not self.__database_client:
            self.__database_client = InfluxDBClient(url=self.__config.influxdb_url,
                                                    token=self.__config.influxdb_token,
                                                    org=self.__config.influxdb_org)
            write_api = self.__database_client.write_api(write_options=SYNCHRONOUS)
        return self.__database_client

    def write_temperature_record(self, point: Point) -> object:
        write_api = self.get_database_client().write_api(write_options=SYNCHRONOUS)
        return write_api.write(self.__config.influxdb_bucket, self.__config.influxdb_org, point)
