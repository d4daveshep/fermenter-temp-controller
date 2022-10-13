import configparser

import requests

from config import ControllerConfig


class TemperatureDatabase:
    def __init__(self, config: ControllerConfig):
        self.__config = config

    def is_server_available(self):
        # run equivalant of curl -sL -I localhost:8086/ping?wait_for_leader=30s
        url = self.__config.influxdb_url + "/ping"
        params = {'wait_for_leader': "30s"}

        # try:
        response = requests.get(url=url, params=params)
        return response.status_code == 204 and \
               response.headers["X-Influxdb-Build"] == "OSS" and \
               response.headers["X-Influxdb-Version"] >= "v2.4"
        # except Exception as exec:
        #     return False
