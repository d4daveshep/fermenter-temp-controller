import zmq as zmq
from zmq.asyncio import Context, Poller

from controller.config import ControllerConfig


class ZmqReceiver:
    def __init__(self, config: ControllerConfig):
        self.config = config

        # configure the socket
        context = Context.instance()
        self.pull = context.socket(zmq.PULL)
        self.pull.connect(self.config.zmq_url)
        self.poller = Poller()
        self.poller.register(self.pull, zmq.POLLIN)

    async def wait_for_a_message(self):
        # await asyncio.sleep(1)
        events = await self.poller.poll()
        if self.pull in dict(events):
            message_received = await self.pull.recv_multipart()
            return message_received



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
