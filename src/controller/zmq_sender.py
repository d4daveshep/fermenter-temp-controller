import zmq as zmq
from zmq.asyncio import Context

from controller.config import ControllerConfig


class ZmqSender:
    def __init__(self, config: ControllerConfig):
        self.config = config

        # configure the socket
        context = Context.instance()
        self.push = context.socket(zmq.PUSH)
        # self.push.bind("tcp://127.0.0.1:5555")
        self.push.bind(self.config.zmq_url)

    async def send_string(self, string_to_send: str):
        await self.push.send_multipart([string_to_send.encode("utf-8")])
