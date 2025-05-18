from configparser import ConfigParser
from pathlib import Path
from typing import Any

from pydantic import AnyUrl, BaseModel, ConfigDict, HttpUrl, ValidationError


class FermenterConfig(BaseModel):
    target_temp: float
    brew_id: str


class InfluxDBConfig(BaseModel):
    url: HttpUrl
    auth_token: str
    org: str
    bucket: str


class ArduinoConfig(BaseModel):
    serial_port: str
    baud_rate: int


class ZmqConfig(BaseModel):
    url: AnyUrl


class GeneralConfig(BaseModel):
    timezone: str


class ControllerConfig(BaseModel):
    model_config = ConfigDict(extra="forbid")

    fermenter: FermenterConfig
    influxdb: InfluxDBConfig
    arduino: ArduinoConfig
    zmq: ZmqConfig
    general: GeneralConfig

    @staticmethod
    def load_config(filename: Path) -> "ControllerConfig":
        """
        Load the specified config .ini file.
        Raise a ValidationError if the config file is incomplete or has validation errors

        Parameters:
        filename (Path): the file path to the config file

        Returns:
        ControllerConfig: a ControllerConfig object
        """
        if not filename.exists():
            raise FileNotFoundError(f"Config file '{filename}' not found")

        # parse the .ini file
        parser: ConfigParser = ConfigParser()
        parser.read(filename)

        # convert to a dict format
        config_dict: dict[str, Any] = {}
        for section in parser.sections():
            section_dict: dict[str, Any] = {k: v for k, v in parser.items(section)}

            # Convert boolean strings
            for key, value in section_dict.items():
                if value.lower() in ("true", "yes", "on", "1"):
                    section_dict[key] = True
                elif value.lower() in ("false", "no", "off", "0"):
                    section_dict[key] = False
                # Try to convert to int if possible
                elif value.isdigit():
                    section_dict[key] = int(value)
                # Try to convert to float if possible
                elif value.replace(".", "", 1).isdigit() and value.count(".") == 1:
                    section_dict[key] = float(value)

            config_dict[section] = section_dict

        # create pydantic model with validation
        try:
            config: ControllerConfig = ControllerConfig.model_validate(config_dict)
            return config
        except ValidationError as e:
            raise ValidationError(f"Configuruation validation failed: {e}")
