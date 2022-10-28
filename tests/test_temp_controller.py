import json
import random
from os.path import exists

import pytest
from serial.serialutil import SerialException

from controller.config import ControllerConfig
from controller.temp_controller import TempController


@pytest.fixture
def valid_config_file():
    filename = "test_valid_config_file.ini"
    assert exists(filename)

    return filename


@pytest.fixture
def valid_config(valid_config_file):
    controller_config = ControllerConfig(valid_config_file)
    assert controller_config

    return controller_config


@pytest.fixture
def temp_controller(valid_config_file):
    controller = TempController(valid_config_file)
    assert controller
    return controller


def test_create_temperature_controller_object(temp_controller):
    controller_config = temp_controller.config
    assert controller_config

    temperature_database = temp_controller.temperature_database
    assert temperature_database.is_server_available()

    assert temp_controller.serial_port_writer is None
    assert temp_controller.serial_port_reader is None


@pytest.mark.asyncio
async def test_open_serial_connection(valid_config):
    serial_port_reader, serial_port_writer = await TempController.open_serial_connection(valid_config.serial_port)
    assert serial_port_writer
    assert serial_port_reader


@pytest.mark.asyncio
async def test_open_serial_connection_with_bad_port_url():
    with pytest.raises(SerialException) as err_info:
        serial_port_reader, serial_port_writer = await TempController.open_serial_connection("/dev/blah_blah")


@pytest.mark.asyncio
async def test_write_float_to_serial_port(temp_controller):
    try:
        temp_controller.serial_port_reader, temp_controller.serial_port_writer = await TempController.open_serial_connection(
            temp_controller.config.serial_port)
        target_temp = random.randrange(10, 30)
        await temp_controller.write_float_to_serial_port(target_temp)
    except Exception as err_info:
        print(err_info)
        assert False


@pytest.mark.asyncio
async def test_read_line_from_serial(temp_controller):
    try:
        temp_controller.serial_port_reader, temp_controller.serial_port_writer = await TempController.open_serial_connection(
            temp_controller.config.serial_port)

        read_string = await temp_controller.read_line_from_serial()
        assert read_string
    except Exception as err_info:
        print(err_info)
        assert False


def test_convert_dict_int_values_to_float():
    int_dict = {"number_1": 1, "number_2": 2, "number_3": 3.33, "string_1": "abc", "bool_1": True}

    float_dict = TempController.convert_dict_int_values_to_float(int_dict)

    assert type(float_dict["number_1"]) == float
    assert float_dict["number_1"] == 1.0
    assert type(float_dict["number_2"]) == float
    assert float_dict["number_2"] == 2.0
    assert type(float_dict["number_3"]) == float
    assert float_dict["number_3"] == 3.33


@pytest.mark.asyncio
async def test_serial_async_write_and_read(temp_controller):
    try:
        temp_controller.serial_port_reader, temp_controller.serial_port_writer = await TempController.open_serial_connection(
            temp_controller.config.serial_port)

        discard_first_line = await temp_controller.read_line_from_serial()

        target_temp_written = random.randrange(10, 30)
        await temp_controller.write_float_to_serial_port(target_temp_written)
        read_string = await temp_controller.read_line_from_serial()
        json_dict = json.loads(read_string)

        json_dict = TempController.convert_dict_int_values_to_float(json_dict)

        target_temp_read = json_dict["target"]

        assert target_temp_written == target_temp_read

    except Exception as err_info:
        print(err_info)
        assert False

def test_process_zmq_message(temp_controller):

    assert False