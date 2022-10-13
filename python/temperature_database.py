import configparser

import requests

from config import ControllerConfig
from influxdb_client import InfluxDBClient


class TemperatureDatabase:
    def __init__(self, config: ControllerConfig):
        self.__database_client = None
        self.__config = config

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
        return self.__database_client
