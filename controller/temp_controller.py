import asyncio
import json
import logging
from asyncio import StreamReader, StreamWriter
from json import JSONDecodeError

import serial_asyncio

from controller.config import ControllerConfig, Settings
from controller.temperature_database import TemperatureDatabase
from controller.zmq_receiver import ZmqReceiver


class TempController:
    def __init__(self, config_filename: str):
        self.logger = None
        self.config = ControllerConfig(config_filename)
        self.temperature_database = TemperatureDatabase(self.config)
        self.zmq_receiver = ZmqReceiver(self.config)
        self.current_target_temp = 0.0

        self.serial_port_writer = None
        self.serial_port_reader = None

        self.configure_logging()

    @classmethod
    async def open_serial_connection(cls, port_url: str) -> (StreamReader, StreamWriter):
        return await serial_asyncio.open_serial_connection(url=port_url, baudrate=115200)

    async def read_line_from_serial(self) -> str:
        line = await self.serial_port_reader.readline()
        self.logger.debug(f"read {line} from serial")
        return str(line, 'utf-8')

    async def write_float_to_serial_port(self, float_num: float) -> None:
        string_to_write = '<' + str(float_num) + '>'
        self.logger.info(f"writing {string_to_write} to serial")
        self.serial_port_writer.write(string_to_write.encode())
        await asyncio.sleep(0)

    @classmethod
    def convert_dict_int_values_to_float(cls, json_dict: dict) -> dict:

        for key, value in json_dict.items():
            # fix and remove some specific fields
            if key == "json-size":  # don't bother writing this to DB
                # del json_dict[key]
                continue
            if type(value) == int:  # covert any ints to floats (e.g. target)
                json_dict[key] = float(value)

        return json_dict

    async def read_temps_from_serial_and_write_to_database(self):

        while True:
            try:

                json_read = await self.read_line_from_serial()
                json_dict = json.loads(json_read)

                data_dict = TempController.convert_dict_int_values_to_float(json_dict)

                # store some key data fields to display on web
                self.current_target_temp = data_dict["target"]
                self.fermenter_temp_now = data_dict["average"]
                self.fermenter_temp_min = data_dict["min"]
                self.fermenter_temp_max = data_dict["max"]
                self.ambient_temp_now = data_dict["ambient"]
                self.current_action = data_dict["action"]
                self.action_reason_code = data_dict["reason-code"]

                point = self.temperature_database.create_point_from_fermenter_json_dict(json_dict)
                self.temperature_database.write_temperature_record(point)

                await asyncio.sleep(0)

            except Exception as err_info:
                print(err_info)

    async def write_target_temp_to_serial_port_if_updated(self):

        while True:
            # check if target_temp needs updating
            # self.logger.debug("checking if target temp needs updating")
            if self.config.target_temp != self.current_target_temp:
                self.logger.info(f"target temp needs updating to {self.config.target_temp}")
                await self.write_float_to_serial_port(self.config.target_temp)
            else:
                # self.logger.debug("target temp did NOT need updating")
                pass
            await asyncio.sleep(5)

    async def receive_and_process_zmq_message(self):

        while True:
            message_received = await self.zmq_receiver.wait_for_a_message()
            string_received = (message_received[0]).decode("utf-8")
            self.logger.debug(f"received zmq message: {string_received}")

            self.process_zmq_message(string_received)

            await asyncio.sleep(0)

    def process_zmq_message(self, json_string: str) -> None:

        try:
            json_dict = json.loads(json_string)

            if "new-target-temp" in json_dict.keys():
                self.config.target_temp = float(json_dict["new-target-temp"])
                self.logger.info(f"set new target temp to: {self.config.target_temp:.1f}")

            if "new-brew-id" in json_dict.keys():
                brew_id = json_dict["new-brew-id"]
                if type(brew_id) == str:
                    self.config.brew_id = json_dict["new-brew-id"]
                    self.logger.info(f"set new brew-id to: {self.config.brew_id}")
                else:
                    raise ValueError(f"new-brew-id '{brew_id}' is not a string")

        except JSONDecodeError as err_info:
            self.logger.error("error processing zmq message: " + str(err_info))
            raise
        except ValueError as err_info:
            self.logger.error("error: processing zmq message: " + str(err_info))
            raise


    def run(self):

        loop = asyncio.get_event_loop()

        try:
            self.serial_port_reader, self.serial_port_writer = loop.run_until_complete(
                self.open_serial_connection(self.config.serial_port))

            discard_first_record = loop.run_until_complete(self.read_line_from_serial())

            reader_task = loop.create_task(self.read_temps_from_serial_and_write_to_database())

            writer_task = loop.create_task(self.write_target_temp_to_serial_port_if_updated())

            receiver_task = loop.create_task(self.receive_and_process_zmq_message())

            loop.run_forever()

        except KeyboardInterrupt:
            print("Stopping loop")
            loop.stop()
        finally:
            print("Closing loop")
            loop.close()

    def configure_logging(self):
        self.logger = logging.getLogger("stdout")
        self.logger.setLevel(logging.DEBUG)
        console_handler = logging.StreamHandler()
        # console_handler.setLevel(logging.DEBUG)
        formatter = logging.Formatter("%(levelname)s: %(asctime)s: %(message)s")
        console_handler.setFormatter(formatter)
        self.logger.addHandler(console_handler)



if __name__ == "__main__":
    # filename = "controller/config-test.ini"

    settings = Settings()

    controller = TempController(settings.config_filename)

    controller.run()
