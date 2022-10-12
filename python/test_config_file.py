# test that the config file is found in the parent directory to this module
import configparser
import random
import tempfile
from os.path import exists

import pytz
from pytz import timezone

import pytest

import config
from config import ConfigError


def test_get_values_from_valid_config_object():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)
    controller_config = config.ControllerConfig(filename)
    assert controller_config

    target_temp = controller_config.target_temp
    assert isinstance(target_temp, float)
    assert target_temp == 21.3

    brew_id = controller_config.brew_id
    assert isinstance(brew_id, str)
    assert brew_id == "00-TEST-v00"

    tz = controller_config.timezone
    assert tz == timezone("Pacific/Auckland")

def test_get_config_fails_if_file_not_exist():
    filename = "./does_not_exist"
    assert not exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = config.ControllerConfig(filename)
        assert controllor_config

    assert err_info.value.args[0] == f"Config file '{filename}' not found"


def test_get_config_fails_if_no_fermenter_section():
    filename = "./test_empty_config_file.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = config.ControllerConfig(filename)
        assert controllor_config

    assert err_info.value.args[0] == "'fermenter' section does not exist in config file"


def test_get_config_fails_if_no_target_temp_in_fermenter_section():
    filename = "./test_config_file_no_target_temp_in_fermenter_section.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = config.ControllerConfig(filename)
        assert controllor_config

        target_temp = controllor_config.target_temp

    assert err_info.value.args[0] == "target_temp not found"


def test_get_config_fails_if_invalid_target_temp():
    filename = "./test_config_file_invalid_target_temp.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = config.ControllerConfig(filename)
        assert controllor_config

        target_temp = controllor_config.target_temp

    assert err_info.value.args[0] == "target_temp 'blah_blah' could not be read as a float"


def test_get_config_file_fails_if_no_brew_id_in_fermenter_section():
    filename = "./test_config_file_no_brew_id_in_fermenter_section.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = config.ControllerConfig(filename)
        assert controllor_config

        brew_id = controllor_config.brew_id

    assert err_info.value.args[0] == "brew_id not found"


def test_get_timezone_from_config():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)

    controller_config = config.ControllerConfig(filename)
    assert controller_config

    nztz = controller_config.timezone
    assert nztz == timezone("Pacific/Auckland")


def test_config_fails_if_no_general_section():
    filename = "./test_config_file_no_general_section.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = config.ControllerConfig(filename)
        assert controller_config

        tz = controller_config.timezone

    assert err_info.value.args[0] == "'general' section not found in config file"

def test_config_defaults_to_auckland_if_no_timezone_in_general_section():
    filename = "./test_config_file_no_timezone_in_general_section.txt"
    assert exists(filename)

    controller_config = config.ControllerConfig(filename)
    assert controller_config

    tz = controller_config.timezone
    assert tz == pytz.timezone("Pacific/Auckland")

def test_config_fails_if_invalid_timezone_general_section():
    filename = "./test_config_file_invalid_timezone_in_general_section.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = config.ControllerConfig(filename)
        assert controller_config

        tz = controller_config.timezone

    assert err_info.value.args[0] == "invalid timezone 'blah_blah' in general section of config file"

