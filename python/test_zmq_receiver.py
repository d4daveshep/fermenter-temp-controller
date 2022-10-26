# I don't know how to test this yet
import json

from config import ControllerConfig

import zmq as zmq
from zmq.asyncio import Context, Poller


class ZmqReceiver:
    def __init__(self, config: ControllerConfig):
        self.config = config

        # configure the socket
        context = Context.instance()
        self.pull = context.socket(zmq.PULL)
        self.pull.connect(self.config.zmq_url)
        self.poller = Poller()
        self.poller.register(self.pull, zmq.POLLIN)


    async def zmq_receiver(self, ctx: Context, url: str) -> None:
        """receive messages with polling"""
        while True:
            events = await poller.poll()
            if pull in dict(events):
                msg = await pull.recv_multipart()
                dict_received = json.loads(msg[0].decode("utf-8"))
                print('received via zmq', dict_received)

                try:
                    target_temp = dict_received["target-temp"]
                    await write_serial_string(writer, target_temp)
                except KeyError as err_info:
                    print(f"{err_info} not received" )



def test_create_zmq_receiver():

    config = ControllerConfig("./test_valid_config_file.txt")

    receiver = ZmqReceiver(config)

    assert receiver

