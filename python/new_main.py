import configparser
from os.path import exists
from pytz import timezone


class ConfigError(Exception):
    pass


class ControllerConfig:
    def __init__(self, filename: str):
        self.timezone = timezone("Pacific/Auckland")
        self.target_temp = None
        if not exists(filename):
            raise ConfigError(f"Config file '{filename}' not found")

        config_parser = self._get_config(filename)
        self.target_temp = self._get_target_temp_from_config(config_parser)
        self.brew_id = self._get_brew_id_from_config(config_parser)

    def _get_config(self, config_filename_location: str):
        try:
            config = configparser.ConfigParser()
            config.read(config_filename_location)
            fermenter_section = config["fermenter"]
            return config
        except KeyError:
            raise ConfigError("'fermenter' section does not exist in config file")

    def _get_target_temp_from_config(self, config: configparser.ConfigParser) -> float:
        try:
            fermenter_section = config["fermenter"]
            target_temp_string = fermenter_section["target_temp"]
            target_temp = float(target_temp_string)
            return target_temp
        except KeyError as err:
            raise ConfigError("target_temp not found")
        except ValueError as err:
            raise ConfigError(f"target_temp '{target_temp_string}' could not be read as a float")

    def _get_brew_id_from_config(self, config: configparser.ConfigParser) -> str:
        try:
            fermenter_section = config["fermenter"]
            brew_id = fermenter_section["brew_id"]
            return brew_id
        except KeyError:
            raise ConfigError("brew_id not found")
