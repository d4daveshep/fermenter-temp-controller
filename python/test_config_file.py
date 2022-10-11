# test that the config file is found in the parent directory to this module
import configparser
import random
import tempfile
from os.path import exists

import pytest

import new_main
from new_main import ConfigError


def test_get_config_object():
    filename = "./test_config_file.txt"
    assert exists(filename)

    controller_config = new_main.ControllerConfig(filename)
    assert controller_config
    target_temp = controller_config.target_temp
    assert isinstance(target_temp, float)
    assert target_temp == 21.3


def test_get_config_fails_if_file_not_exist():
    filename = "./does_not_exist"
    assert not exists(filename)

    with pytest.raises(ConfigError) as err_info:
        config = new_main.get_config(filename)


def test_get_config_fails_if_no_fermenter_section():
    filename = "./test_empty_config_file.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        config = new_main.get_config(filename)


def test_get_config_fails_if_no_target_temp_in_fermenter_section():
    filename = "./test_config_file_no_target_temp_in_fermenter_section.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        config = new_main.get_config(filename)
        target_temp = new_main.get_target_temp_from_config(config)

    assert err_info.value.args[0] == "target_temp not found"


def test_get_config_fails_if_invalid_target_temp():
    filename = "./test_config_file_invalid_target_temp.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        config = new_main.get_config(filename)
        target_temp = new_main.get_target_temp_from_config(config)

    assert err_info.value.args[0] == "target_temp 'blah_blah' could not be read as a float"


def test_get_target_temp_from_config_file():
    filename = "./test_config_file.txt"
    assert exists(filename)

    config = new_main.get_config(filename)
    target_temp = new_main.get_target_temp_from_config(config)
    assert isinstance(target_temp, float)
    assert target_temp == 21.3


def test_get_config_file_fails_if_no_brew_id_in_fermenter_section():
    filename = "./test_config_file_no_brew_id_in_fermenter_section.txt"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        config = new_main.get_config(filename)
        brew_id = new_main.get_brew_id_from_config(config)

    assert err_info.value.args[0] == "brew_id not found"


def test_get_brew_id_from_config_file():
    filename = "./test_config_file.txt"
    assert exists(filename)

    config = new_main.get_config(filename)
    brew_id = new_main.get_brew_id_from_config(config)
    assert brew_id == "00-TEST-v00"

# def test_get_target_temp_from_config(config_filename_location):
#     config = new_main.get_config(config_filename_location)
#     target_temp = new_main.get_target_temp_from_config(config)
#     assert
#     try:
#         target_temp = float(target_temp_string)
#     except ValueError as error:
#         assert False, "target_temp is not a floating number"
#
