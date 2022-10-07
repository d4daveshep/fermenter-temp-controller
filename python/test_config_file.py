# test that the config file is found in the parent directory to this module
import configparser
from os.path import exists

import pytest


@pytest.fixture
def config_filename_location():
    return "../config.txt"


@pytest.fixture
def config(config_filename_location):
    config = configparser.ConfigParser()
    config.read(config_filename_location)
    return config


def test_config_file_exists(config_filename_location):
    file_exists = exists(config_filename_location)
    assert file_exists


def test_config_file_contains_valid_target_temp(config):
    assert "fermenter" in config.sections()
    fermenter_section = config["fermenter"]

    assert "target_temp" in fermenter_section

    target_temp_string = fermenter_section["target_temp"]
    try:
        target_temp = float(target_temp_string)
    except ValueError as error:
        assert False, "target_temp is not a floating number"

def test_config_file_contains_brew_id(config):
    assert "fermenter" in config.sections()
    fermenter_section = config["fermenter"]

    assert "brew_id" in fermenter_section, "'brew_id' not in configfile [fermenter] section"
    brew_id = fermenter_section["brew_id"]
    assert brew_id, "'brew_id' is empty string"

