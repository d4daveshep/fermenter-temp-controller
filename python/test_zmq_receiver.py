# I don't know how to test this yet
from os.path import exists

import pytest
import zmq as zmq
from zmq.asyncio import Context, Poller

from config import ControllerConfig


class ZmqReceiver:
    def __init__(self, config: ControllerConfig):
        self.config = config

        # configure the socket
        context = Context.instance()
        self.pull = context.socket(zmq.PULL)
        self.pull.connect(self.config.zmq_url)
        self.poller = Poller()
        self.poller.register(self.pull, zmq.POLLIN)

    # async def zmq_receiver(self, ctx: Context, url: str) -> None:
    #     """receive messages with polling"""
    #     while True:
    #         events = await poller.poll()
    #         if pull in dict(events):
    #             msg = await pull.recv_multipart()
    #             dict_received = json.loads(msg[0].decode("utf-8"))
    #             print('received via zmq', dict_received)
    #
    #             try:
    #                 target_temp = dict_received["target-temp"]
    #                 await write_serial_string(writer, target_temp)
    #             except KeyError as err_info:
    #                 print(f"{err_info} not received")


@pytest.fixture
def valid_config_file():
    filename = "./test_valid_config_file.txt"
    assert exists(filename)
    return filename


def test_create_zmq_receiver(valid_config_file):
    config = ControllerConfig("./test_valid_config_file.txt")
    receiver = ZmqReceiver(config)
    assert receiver


@pytest.fixture
def receiver(valid_config_file):
    config = ControllerConfig(valid_config_file)
    receiver = ZmqReceiver(config)
    assert receiver
    return receiver


@pytest.mark.asyncio
async def test_receive_json(receiver):
    assert False
