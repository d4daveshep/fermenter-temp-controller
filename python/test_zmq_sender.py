import json
import random
from os.path import exists

import pytest

from config import ControllerConfig
from zmq_sender import ZmqSender


@pytest.fixture
def valid_config_file():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)
    return filename


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
    config = ControllerConfig("./test_valid_config_file.txt")
    try:
        sender = ZmqSender(config)
        assert sender
    except Exception as err_info:
        assert False



@pytest.mark.asyncio
async def test_send_json(sender):
    target_temp = random.randrange(10, 30)
    string_to_send = json.dumps({"target-temp": target_temp})
    try:
        await sender.send_string(string_to_send)
    except Exception as err_info:
        assert False
