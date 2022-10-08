import configparser
from os.path import exists


def get_config(config_filename_location):
    assert exists(config_filename_location)
    config = configparser.ConfigParser()
    config.read(config_filename_location)

    assert "fermenter" in config.sections()
    fermenter_section = config["fermenter"]

    assert "target_temp" in fermenter_section
    target_temp_string = fermenter_section["target_temp"]
    try:
        target_temp = float(target_temp_string)
    except ValueError as error:
        assert False, "target_temp is not a floating number"

    return config


def get_target_temp_from_config(config_filename_location) -> float:
    config = get_config(config_filename_location)
    fermenter_section = config["fermenter"]
    target_temp_string = fermenter_section["target_temp"]
    target_temp = float(target_temp_string)
    return target_temp


