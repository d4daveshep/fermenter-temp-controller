# test that the config file is found in the parent directory to this module
import configparser
import random
import tempfile
from os.path import exists

import pytest

import new_main

#
# @pytest.fixture
# def config_filename_location():
#     return "../config.txt"
#
#
# @pytest.fixture
# def config(config_filename_location):
#     config = configparser.ConfigParser()
#     config.read(config_filename_location)
#     return config
#
#
# def test_config_file_exists(config_filename_location):
#     file_exists = exists(config_filename_location)
#     assert file_exists
#
#
# def test_config_file_contains_valid_target_temp(config):
#     assert "fermenter" in config.sections()
#     fermenter_section = config["fermenter"]
#
#     assert "target_temp" in fermenter_section
#
#     target_temp_string = fermenter_section["target_temp"]
#     try:
#         target_temp = float(target_temp_string)
#     except ValueError as error:
#         assert False, "target_temp is not a floating number"
#
# def test_config_file_contains_brew_id(config):
#     assert "fermenter" in config.sections()
#     fermenter_section = config["fermenter"]
#
#     assert "brew_id" in fermenter_section, "'brew_id' not in configfile [fermenter] section"
#     brew_id = fermenter_section["brew_id"]
#     assert brew_id, "'brew_id' is empty string"

def test_get_config_fails_if_file_not_exist():
    filename = "./does_not_exist"
    assert not exists(filename)

    with pytest.raises(AssertionError) as err_info:
        config = new_main.get_config(filename)

def test_get_config_fails_if_no_fermenter_section():
    filename = "./test_empty_config_file.txt"
    assert exists(filename)

    with pytest.raises(AssertionError) as err_info:
        config = new_main.get_config(filename)
        # fermenter_section = config["fermenter"]

def test_get_config_fails_if_no_target_temp_in_fermenter_section():
    filename = "./test_config_file_no_target_temp_in_fermenter_section.txt"
    assert exists(filename)

    with pytest.raises(AssertionError) as err_info:
        config = new_main.get_config(filename)

def test_get_config_fails_if_invalid_target_temp():
    filename = "./test_config_file_invalid_target_temp.txt"
    assert exists(filename)

    with pytest.raises(AssertionError) as err_info:
        config = new_main.get_config(filename)

def test_get_target_temp_from_config_file():
    filename = "./test_config_file.txt"
    assert exists(filename)

    target_temp = new_main.get_target_temp_from_config(filename)
    assert isinstance(target_temp, float)
    assert target_temp == 21.3


# def test_get_target_temp_from_config(config_filename_location):
#     config = new_main.get_config(config_filename_location)
#     target_temp = new_main.get_target_temp_from_config(config)
#     assert
#     try:
#         target_temp = float(target_temp_string)
#     except ValueError as error:
#         assert False, "target_temp is not a floating number"
#



