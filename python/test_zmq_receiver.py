# I don't know how to test this yet
from os.path import exists

import pytest

from config import ControllerConfig
from zmq_receiver import ZmqReceiver


@pytest.fixture
def valid_config_file():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)
    return filename


@pytest.fixture
def receiver(valid_config_file):
    config = ControllerConfig(valid_config_file)
    receiver = ZmqReceiver(config)
    assert receiver
    return receiver


def test_create_zmq_receiver(valid_config_file):
    config = ControllerConfig("./test_valid_config_file.txt")
    receiver = ZmqReceiver(config)
    assert receiver


@pytest.mark.asyncio
async def test_receive_json(receiver):
    json_string = await receiver.wait_for_a_message()
    print(f"received {json_string}")
    assert json_string
