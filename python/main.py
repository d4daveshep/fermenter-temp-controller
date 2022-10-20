import asyncio
import glob
import json
from asyncio import StreamWriter, StreamReader

import serial
import serial_asyncio
from serial import SerialException, Serial


# def loop(config: ControllerConfig):
#     serial_port = get_serial_port()
#
#     target_temp = config.target_temp
#     while (True):
#
#         if config.target_temp != target_temp:
#             # write new target_temp to arduino
#             pass
#

def write_float_to_serial(serial_port: Serial, num: float):
    string_to_write = '<' + str(num) + '>'
    serial_port.write(string_to_write.encode())


def read_json_from_serial(serial_port: Serial) -> dict:
    line = serial_port.readline()
    return json.loads(line.decode("utf-8"))


def get_serial_port() -> Serial:
    # find the serial port
    ttylist = glob.glob("/dev/ttyACM*")
    if len(ttylist) == 0:
        msg = "No Arduino devices found at /dev/ttyACM*"
        raise SystemExit("*** ERROR *** " + msg)

    if len(ttylist) > 1:
        msg = "WARNING: Multiple Arduino devices found at /dev/ttyACM*, using first device"

    tty = ttylist[0]

    # open the serial port
    try:
        port = serial.Serial(tty, 115200)
    except SerialException:
        raise SystemExit("*** ERROR *** Couldn't open serial port")

    # sleep for 30 secs to allow arduino to reboot after serial port open
    # time.sleep(30)

    return port


async def open_serial_connection(port_url: str) -> (StreamReader, StreamWriter):
    return await serial_asyncio.open_serial_connection(url=port_url, baudrate=115200)


async def write_float_to_serial_port(serial_port_writer: StreamWriter, float_num: float) -> None:
    string_to_write = '<' + str(float_num) + '>'
    print(f"writing {string_to_write} to serial")
    result = serial_port_writer.write(string_to_write.encode())
    await asyncio.sleep(0)


async def read_line_from_serial(serial_port_reader: StreamReader) -> str:
    line = await serial_port_reader.readline()
    return str(line, 'utf-8')
