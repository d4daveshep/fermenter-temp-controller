# test that the config file is found in the parent directory to this module
from os import getcwd
from os.path import exists

import pytest
import pytz
from pytz import timezone

from controller.config import ControllerConfig, ConfigError


def test_get_target_temp_config_file():
    filename = "test_valid_config_file.ini"
    assert exists(filename)
    controller_config = ControllerConfig(filename)
    assert controller_config

    target_temp = controller_config.target_temp
    assert isinstance(target_temp, float)
    assert target_temp == 21.3


def test_get_brew_id_from_config_file():
    filename = "test_valid_config_file.ini"
    assert exists(filename)
    controller_config = ControllerConfig(filename)
    assert controller_config

    brew_id = controller_config.brew_id
    assert isinstance(brew_id, str)
    assert brew_id == "00-TEST-v00"


def test_get_config_fails_if_file_not_exist():
    filename = "./does_not_exist"
    assert not exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = ControllerConfig(filename)
        assert controllor_config

    cwd = getcwd()
    assert err_info.value.args[0] == f"Config file '{filename}' not found in {cwd}"


def test_get_config_fails_if_no_fermenter_section():
    filename = "test_config_file_no_fermenter_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = ControllerConfig(filename)
        target_temp = controllor_config.target_temp

    assert err_info.value.args[0] == "'fermenter' section not found in config file"


def test_get_config_fails_if_no_target_temp_in_fermenter_section():
    filename = "test_config_file_no_target_temp_in_fermenter_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = ControllerConfig(filename)
        assert controllor_config

        target_temp = controllor_config.target_temp

    assert err_info.value.args[0] == "target_temp not found"


def test_get_config_fails_if_invalid_target_temp():
    filename = "test_config_file_invalid_target_temp.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = ControllerConfig(filename)
        assert controllor_config

        target_temp = controllor_config.target_temp

    assert err_info.value.args[0] == "target_temp 'blah_blah' could not be read as a float"


def test_get_config_file_fails_if_no_brew_id_in_fermenter_section():
    filename = "test_config_file_no_brew_id_in_fermenter_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controllor_config = ControllerConfig(filename)
        assert controllor_config

        brew_id = controllor_config.brew_id

    assert err_info.value.args[0] == "'brew_id' not found in fermenter section in config file"


def test_get_timezone_from_config():
    filename = "test_valid_config_file.ini"
    assert exists(filename)

    controller_config = ControllerConfig(filename)
    assert controller_config

    nztz = controller_config.timezone
    assert nztz == timezone("Pacific/Auckland")


def test_config_fails_if_no_general_section():
    filename = "test_config_file_no_general_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)
        assert controller_config

        tz = controller_config.timezone

    assert err_info.value.args[0] == "'general' section not found in config file"


def test_config_defaults_to_auckland_if_no_timezone_in_general_section():
    filename = "test_config_file_no_timezone_in_general_section.ini"
    assert exists(filename)

    controller_config = ControllerConfig(filename)
    assert controller_config

    tz = controller_config.timezone
    assert tz == pytz.timezone("Pacific/Auckland")


def test_config_fails_if_invalid_timezone_general_section():
    filename = "test_config_file_invalid_timezone_in_general_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)
        assert controller_config

        tz = controller_config.timezone

    assert err_info.value.args[0] == "invalid timezone 'blah_blah' in general section of config file"


def test_get_influxdb_credentials_from_config():
    filename = "test_valid_config_file.ini"
    assert exists(filename)

    controller_config = ControllerConfig(filename)
    assert controller_config

    assert controller_config.influxdb_url == "http://localhost:8086"
    assert controller_config.influxdb_token == "my-super-secret-auth-token"
    assert controller_config.influxdb_org == "daveshep.net"
    assert controller_config.influxdb_bucket == "temp-test"


def test_config_fails_if_no_influxdb_section():
    filename = "test_config_file_no_influxdb_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'influxdb' section not found in config file"


def test_config_fails_if_no_auth_token_in_influxdb_section():
    filename = "test_config_file_no_auth_token_in_influxdb_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'auth_token' not found in influxdb section in config file"


def test_config_fails_if_no_url_in_influxdb_section():
    filename = "test_config_file_no_url_in_influxdb_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'url' not found in influxdb section in config file"


def test_config_fails_if_invalid_url_in_influxdb_section():
    filename = "test_config_file_invalid_url_in_influxdb_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "URL 'blah_blah' is not valid URL in influxdb section in config file"


def test_config_fails_if_no_org_in_influxdb_section():
    filename = "test_config_file_no_org_in_influxdb_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'org' not found in influxdb section in config file"


def test_config_fails_if_no_bucket_in_influxdb_section():
    filename = "test_config_file_no_bucket_in_influxdb_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'bucket' not found in influxdb section in config file"


def test_get_serial_port_from_config():
    filename = "test_valid_config_file.ini"
    assert exists(filename)

    controller_config = ControllerConfig(filename)
    assert controller_config

    assert controller_config.serial_port == "/dev/ttyACM0"


def test_config_fails_if_no_arduino_section():
    filename = "test_config_file_no_arduino_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'arduino' section not found in config file"


def test_get_zmq_url_from_config():
    filename = "test_valid_config_file.ini"
    assert exists(filename)

    controller_config = ControllerConfig(filename)
    assert controller_config

    assert controller_config.zmq_url == "tcp://127.0.0.1:5555"


def test_config_fails_if_no_zmq_section():
    filename = "test_config_file_no_zmq_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'zmq' section not found in config file"


def test_config_fails_if_no_url_in_zmq_section():
    filename = "test_config_file_no_url_in_zmq_section.ini"
    assert exists(filename)

    with pytest.raises(ConfigError) as err_info:
        controller_config = ControllerConfig(filename)

    assert err_info.value.args[0] == "'url' not found in zmq section in config file"


# def test_config_fails_if_invalid_url_in_zmq_section():
#     filename = "test_config_file_invalid_url_in_zmq_section.ini"
#     assert exists(filename)
#
#     with pytest.raises(ConfigError) as err_info:
#         controller_config = ControllerConfig(filename)
#
#     assert err_info.value.args[0] == "URL 'blah_blah' is not valid URL in zmq section in config file"
