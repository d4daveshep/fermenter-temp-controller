import random
from os.path import exists

import pytest
from serial.serialutil import SerialException

import config
from temp_controller import TempController


@pytest.fixture
def valid_config_file():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)

    return filename


@pytest.fixture
def valid_config(valid_config_file):
    controller_config = config.ControllerConfig(valid_config_file)
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

    assert False  # TODO finish this


@pytest.mark.asyncio
async def test_open_serial_connection(temp_controller, valid_config):
    serial_port_reader, serial_port_writer = await temp_controller.open_serial_connection(valid_config.serial_port)


@pytest.mark.asyncio
async def test_open_serial_connection_with_bad_port_url(temp_controller):
    with pytest.raises(SerialException) as err_info:
        serial_port_reader, serial_port_writer = await temp_controller.open_serial_connection("/dev/blah_blah")


@pytest.mark.asyncio
async def test_write_async_serial_string(temp_controller):
    target_temp = random.randrange(10, 30)
    await temp_controller.write_float_to_serial_port(target_temp)


@pytest.mark.asyncio
async def test_read_line_from_serial(temp_controller):
    read_string = await temp_controller.read_line_from_serial()
    assert read_string

@pytest.mark.asyncio
async def test_serial_async_write_and_read(temp_controller):

    target_temp_written = random.randrange(10, 30)
    await temp_controller.write_float_to_serial_port(target_temp_written)
    read_string = await temp_controller.read_line_from_serial()

    target_temp_read = temp_controller.get_target_temp_from_serial_string(read_string)

    assert target_temp_written == target_temp_read