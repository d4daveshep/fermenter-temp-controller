import configparser


# from os.path import exists


class ConfigError(Exception):
    pass


def get_config(config_filename_location):
    try:
        config = configparser.ConfigParser()
        config.read(config_filename_location)
        fermenter_section = config["fermenter"]
        return config
    except KeyError:
        raise ConfigError("'fermenter' section does not exist")
    # except Exception as err:
    #     raise ConfigError(err)


def get_target_temp_from_config(config: configparser.ConfigParser) -> float:
    try:
        fermenter_section = config["fermenter"]
        target_temp_string = fermenter_section["target_temp"]
        target_temp = float(target_temp_string)
        return target_temp
    except KeyError as err:
        raise ConfigError("target_temp not found")
    except ValueError as err:
        raise ConfigError(f"target_temp '{target_temp_string}' could not be read as a float")


def get_brew_id_from_config(config: configparser.ConfigParser) -> str:
    try:
        fermenter_section = config["fermenter"]
        brew_id = fermenter_section["brew_id"]
        return brew_id
    except KeyError:
        raise ConfigError("brew_id not found")


class ControllerConfig:
    def __init__(self, filename: str):
        pass
