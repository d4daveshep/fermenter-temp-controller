# main.py
import asyncio
import serial_asyncio
from asyncio import Queue, StreamReader, StreamWriter, Task


async def read_from_arduino(reader: StreamReader, db_queue: Queue) -> None:
    """Continuously read from Arduino serial port and queue data for database"""
    while True:
        try:
            data: bytes = await reader.readline()
            decoded_data: str = data.decode("utf-8").strip()
            if decoded_data:
                await db_queue.put(decoded_data)
                await asyncio.sleep(0.5)
        except Exception as e:
            print(f"Error reading from Arduino: {e}")
            await asyncio.sleep(0.5)


async def write_to_database(db_queue: Queue) -> None:
    """Process database writes from the queue"""
    while True:
        try:
            data: str = await db_queue.get()
            # Your database write logic here
            await write_data_to_db(data)
            db_queue.task_done()
        except Exception as e:
            print(f"Error writing to database: {e}")


async def handle_arduino_commands(writer: StreamWriter, command_queue: Queue) -> None:
    """Process commands that need to be sent to Arduino"""
    while True:
        try:
            command: str = await command_queue.get()
            writer.write(f"{command}\n".encode("utf-8"))
            await writer.drain()
            command_queue.task_done()
            print(f"Sent arduino command: {command}")
        except Exception as e:
            print(f"Error writing to Arduino: {e}")


async def write_data_to_db(data: str) -> None:
    """Placeholder for your database write implementation"""
    # Your actual database code here
    await asyncio.sleep(0.01)  # Simulate async DB operation
    print(f"Wrote to DB: {data}")


async def main() -> None:
    # Create queues for inter-task communication
    db_queue: Queue = Queue()
    command_queue: Queue = Queue()

    # Open serial connection to Arduino
    reader: StreamReader
    writer: StreamWriter
    reader, writer = await serial_asyncio.open_serial_connection(
        url="/dev/ttyUSB0",  # Adjust for your Arduino port
        baudrate=9600,
    )

    # Create and run all tasks concurrently
    tasks: list[Task] = [
        asyncio.create_task(read_from_arduino(reader, db_queue)),
        asyncio.create_task(write_to_database(db_queue)),
        asyncio.create_task(handle_arduino_commands(writer, command_queue)),
    ]

    try:
        # Example: Add a command to the queue
        await command_queue.put("LED_ON")

        # Run all tasks concurrently
        await asyncio.gather(*tasks)

    except KeyboardInterrupt:
        print("Shutting down...")
        for task in tasks:
            task.cancel()
        writer.close()
        await writer.wait_closed()


if __name__ == "__main__":
    asyncio.run(main())
