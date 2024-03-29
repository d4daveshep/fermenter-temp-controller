# I don't know how to test this yet
import json
import random
from os.path import exists

import pytest

from controller.config import ControllerConfig
from controller.zmq_receiver import ZmqReceiver
from controller.zmq_sender import ZmqSender


@pytest.fixture
def valid_config_file():
    filename = "test_valid_config_file.ini"
    assert exists(filename)
    return filename


@pytest.fixture
def receiver(valid_config_file):
    config = ControllerConfig(valid_config_file)
    receiver = ZmqReceiver(config)
    assert receiver
    return receiver


def test_create_zmq_receiver(valid_config_file):
    config = ControllerConfig("test_valid_config_file.ini")
    receiver = ZmqReceiver(config)
    assert receiver


@pytest.fixture
def sender(valid_config_file):
    config = ControllerConfig(valid_config_file)
    try:
        sender = ZmqSender(config)
        assert sender
        return sender
    except Exception as err_info:
        assert False


def test_create_zmq_sender(valid_config_file):
    config = ControllerConfig("test_valid_config_file.ini")
    try:
        sender = ZmqSender(config)
        assert sender
    except Exception as err_info:
        assert False


@pytest.mark.asyncio
async def test_send_and_receive_json(valid_config_file, receiver):
    config = ControllerConfig(valid_config_file)

    target_temp = random.randrange(10, 30)
    string_to_send = json.dumps({"new-target-temp": target_temp})
    try:
        sender = ZmqSender(config)
        await sender.send_string(string_to_send)

        message_received = await receiver.wait_for_a_message()
        string_received = (message_received[0]).decode("utf-8")
        assert string_to_send == string_received

    except Exception as err_info:
        assert False

