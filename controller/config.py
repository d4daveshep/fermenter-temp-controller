import configparser
from os.path import exists

import pytz
import validators as validators


class ConfigError(Exception):
    pass


class ControllerConfig:
    def __init__(self, filename: str):
        if not exists(filename):
            raise ConfigError(f"Config file '{filename}' not found")

        config_parser = self._get_config(filename)
        self.target_temp = self._get_target_temp_from_config(config_parser)
        self.brew_id = self._get_brew_id_from_config(config_parser)

        self.influxdb_url = self._get_influxdb_url_from_config(config_parser)
        self.influxdb_token = self._get_influxdb_auth_token_from_config(config_parser)
        self.influxdb_org = self._get_influxdb_org_from_config(config_parser)
        self.influxdb_bucket = self._get_influxdb_bucket_from_config(config_parser)

        self.serial_port = self._get_serial_port_from_config(config_parser)


        self.timezone = self._get_timezone_from_config(config_parser)

        self.zmq_url = self._get_zmq_url_from_config(config_parser)



    def _get_config(self, config_filename_location: str):
        try:
            config = configparser.ConfigParser()
            config.read(config_filename_location)
            # fermenter_section = config["fermenter"]
            return config
        except configparser.Error as err:
            raise ConfigError(err)

    def _get_fermenter_section_from_config(self, config_parser: configparser.ConfigParser) -> configparser.SectionProxy:
        try:
            return config_parser["fermenter"]
        except KeyError:
            raise ConfigError("'fermenter' section not found in config file")


    def _get_target_temp_from_config(self, config: configparser.ConfigParser) -> float:
        try:
            fermenter_section = self._get_fermenter_section_from_config(config)
            target_temp_string = fermenter_section["target_temp"]
            return float(target_temp_string)
        except KeyError as err:
            raise ConfigError("target_temp not found")
        except ValueError as err:
            raise ConfigError(f"target_temp '{target_temp_string}' could not be read as a float")

    def _get_brew_id_from_config(self, config: configparser.ConfigParser) -> str:
        try:
            fermenter_section = self._get_fermenter_section_from_config(config)
            brew_id = fermenter_section["brew_id"]
            return brew_id
        except KeyError:
            raise ConfigError("'brew_id' not found in fermenter section in config file")

    def _get_general_section_from_config(self, config_parser: configparser.ConfigParser) -> configparser.SectionProxy:
        try:
            return config_parser["general"]
        except KeyError:
            raise ConfigError("'general' section not found in config file")

    def _get_timezone_from_config(self, config: configparser.ConfigParser) -> pytz.timezone:
        try:
            general_section = self._get_general_section_from_config(config)
            if "timezone" in general_section:
                tz_string = general_section["timezone"]
            else:
                tz_string = "Pacific/Auckland"
            return pytz.timezone(tz_string)
        except pytz.exceptions.Error as err:
            raise ConfigError(f"invalid timezone {err} in general section of config file")

    def _get_influxdb_section_from_config(self, config_parser: configparser.ConfigParser) -> configparser.SectionProxy:
        try:
            return config_parser["influxdb"]
        except KeyError:
            raise ConfigError("'influxdb' section not found in config file")

    def _get_influxdb_auth_token_from_config(self, config: configparser.ConfigParser) -> str:
        try:
            influxdb_section = self._get_influxdb_section_from_config(config)
            return influxdb_section["auth_token"]
        except KeyError:
            raise ConfigError("'auth_token' not found in influxdb section in config file")

    def _get_influxdb_org_from_config(self, config: configparser.ConfigParser) -> str:
        try:
            influxdb_section = self._get_influxdb_section_from_config(config)
            return influxdb_section["org"]
        except KeyError:
            raise ConfigError("'org' not found in influxdb section in config file")

    def _get_influxdb_bucket_from_config(self, config: configparser.ConfigParser) -> str:
        try:
            influxdb_section = self._get_influxdb_section_from_config(config)
            return influxdb_section["bucket"]
        except KeyError:
            raise ConfigError("'bucket' not found in influxdb section in config file")

    def _get_influxdb_url_from_config(self, config: configparser.ConfigParser) -> str:
        try:
            influxdb_section = self._get_influxdb_section_from_config(config)
            url = influxdb_section["url"]
            if not validators.url(url):
                raise ConfigError(f"URL '{url}' is not valid URL in influxdb section in config file")
            return url
        except KeyError:
            raise ConfigError("'url' not found in influxdb section in config file")

    def _get_serial_port_from_config(self, config: configparser.ConfigParser) -> str:
        try:
            arduino_section = self._get_arduino_section_from_config(config)
            serial_port = arduino_section["serial_port"]
            return serial_port
        except KeyError:
            raise ConfigError("'serial_port' not found in arduino section in config file")

    def _get_arduino_section_from_config(self, config_parser: configparser.ConfigParser) -> configparser.SectionProxy:
        try:
            return config_parser["arduino"]
        except KeyError:
            raise ConfigError("'arduino' section not found in config file")

    def _get_zmq_url_from_config(self, config_parser: configparser.ConfigParser) -> str:
        try:
            zmq_section = self._get_zmq_section_from_config(config_parser)
            zmq_url = zmq_section["url"]
            # if not validators.url(zmq_url):
            #     raise ConfigError(f"URL '{zmq_url}' is not valid URL in influxdb section in config file")
            return zmq_url
        except KeyError:
            raise ConfigError("'url' not found in zmq section in config file")

    def _get_zmq_section_from_config(self, config_parser: configparser.ConfigParser) -> configparser.SectionProxy:
        try:
            return config_parser["zmq"]
        except KeyError:
            raise ConfigError("'zmq' section not found in config file")
