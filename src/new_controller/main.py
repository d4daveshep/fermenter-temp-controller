# main.py
import asyncio
import json
import logging
from asyncio import Queue, StreamReader, StreamWriter, Task
from contextlib import asynccontextmanager
from typing import Any

import serial_asyncio
import uvicorn
from fastapi import FastAPI
from pydantic_core import ValidationError

from controller.temperature import TemperatureReading
from mock_arduino_controller.mock_serial_connection import patch_serial_connection

# TODO: Move mocking flag into a config or environment variable
MOCK_SERIAL_CONNECTION: bool = False

# Define global queues for inter-task communication
db_queue: Queue
command_queue: Queue

# Create global logger
logger = logging.getLogger(__name__)


async def read_from_arduino(
    reader: StreamReader, db_queue: Queue, sleep_interval: float = 0.1
) -> None:
    """Continuously read from Arduino serial port and queue data for database"""
    logging.debug("Starting read_from_arduino task")
    while True:
        try:
            data: bytes = await reader.readline()
            decoded_data: str = data.decode("utf-8").strip()
            logging.debug(f"Data read from Arduino: {decoded_data}")
            if decoded_data:
                temp_reading: TemperatureReading = (
                    TemperatureReading.model_validate_json(decoded_data)
                )
                logging.info(f"TemperatureReading: {temp_reading}")
                await db_queue.put(temp_reading)
                await asyncio.sleep(sleep_interval)
        except ValidationError as e:
            logging.warning(f"ValidationError in data from Arduino: {e}")
        except Exception as e:
            logging.error(f"Error reading from Arduino: {e}")
            await asyncio.sleep(sleep_interval)


async def write_to_database(db_queue: Queue) -> None:
    """Process database writes from the queue"""
    while True:
        try:
            # FIXME: think this should be a TemperatureReading object
            data: str = await db_queue.get()
            # Your database write logic here
            await write_data_to_db(data)
            db_queue.task_done()
        except Exception as e:
            logging.error(f"Error writing to database: {e}")


async def handle_arduino_commands(writer: StreamWriter, command_queue: Queue) -> None:
    """Process commands that need to be sent to Arduino"""
    while True:
        try:
            command: str = await command_queue.get()
            writer.write(f"{command}\n".encode("utf-8"))
            await writer.drain()
            command_queue.task_done()
            logging.info(f"Sent arduino command: {command}")
        except Exception as e:
            logging.error(f"Error writing to Arduino: {e}")


async def write_data_to_db(data: str) -> None:
    """Placeholder for your database write implementation"""
    # Your actual database code here
    await asyncio.sleep(0.01)  # Simulate async DB operation
    logging.info(f"Wrote to DB: {data}")


async def arduino_serial_handler() -> None:
    """
    Handle Arduino serial communication asynchronously
    """
    global db_queue, command_queue

    logging.info("Starting the Arduino serial handler")

    # Create queues for inter-task communication
    db_queue = Queue()
    command_queue = Queue()

    sleep_interval: float = 0.1  # Default
    if MOCK_SERIAL_CONNECTION:
        # Patch the serial connection to use mocked serial connection
        patched: dict[str, Any] = patch_serial_connection()
        # sleep_interval = 10.0  # to simulate the physical Arduino
        pass

    # Open serial connection to Arduino
    try:
        reader: StreamReader
        writer: StreamWriter
        reader, writer = await serial_asyncio.open_serial_connection(
            url="/dev/ttyACM0",  # Adjust for your Arduino port
            baudrate=115200,
        )
    except Exception as e:
        logger.error(f"Failed to connect to Arduino: {e}")
        return

    # Create and run all tasks concurrently
    tasks: list[Task] = [
        asyncio.create_task(read_from_arduino(reader, db_queue, sleep_interval)),
        asyncio.create_task(write_to_database(db_queue)),
        asyncio.create_task(handle_arduino_commands(writer, command_queue)),
    ]

    try:
        # Run all tasks concurrently
        await asyncio.gather(*tasks)

    except asyncio.CancelledError:
        logging.info("Arduino tasks cancelled")
        for task in tasks:
            task.cancel()
        writer.close()
        await writer.wait_closed()


# Lifespan manager for FastAPI
@asynccontextmanager
async def lifespan(app: FastAPI):
    # wait for Arduino connection to establish
    if not MOCK_SERIAL_CONNECTION:
        await asyncio.sleep(10)

    # Start Arduino handler as background task
    arduino_task: Task = asyncio.create_task(arduino_serial_handler())

    yield

    # Cleanup on shutdown
    arduino_task.cancel()
    try:
        await arduino_task
    except asyncio.CancelledError:
        pass


# Create the FastAPI app with lifespan
app: FastAPI = FastAPI(lifespan=lifespan)


@app.get("/")
async def root():
    return {"message": "Arduino Controller API"}


async def run_server():
    """
    Run the FastAPI server
    """
    logging.basicConfig(level=logging.DEBUG)
    logging.info(
        "Starting the FastAPI app and handling Arduino communication in background"
    )

    config = uvicorn.Config(app, host="0.0.0.0", port=8000, log_level="info")
    server = uvicorn.Server(config)
    await server.serve()


if __name__ == "__main__":
    MOCK_SERIAL_CONNECTION = True
    asyncio.run(run_server())
