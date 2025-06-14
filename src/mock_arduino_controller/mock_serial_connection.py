from typing import Any, Generator
from unittest.mock import AsyncMock, MagicMock, patch


from mock_arduino_controller.mock_temperature import mock_arduino_output_generator


def patch_serial_connection() -> dict[str, Any]:
    """
    Patch the serial connection for mocking serial read/write operations.
    Return a dictionary of mocked objects
    """
    mock_reader: AsyncMock = AsyncMock()
    mock_writer: AsyncMock = MagicMock()

    # Mock the close method and other writer methods
    mock_writer.close = MagicMock()
    mock_writer.write = MagicMock()
    mock_writer.drain = (
        AsyncMock()
    )  # drain() is async and waits for write buffer to empty

    # Configure the writer to store data written
    written_data: list[bytes] = []

    def store_writes(data: bytes) -> Any:
        written_data.append(data)
        return MagicMock.return_value

    mock_writer.write.side_effect = store_writes

    # convert the float value from the last written data
    def latest_target_temp() -> float:
        temp: float = 20.0
        if written_data:
            temp_str: str = written_data[-1].decode()
            temp = float(temp_str[1:-1])
        return temp

    # Configure readline to return different responses
    mock_reader.readline.side_effect = mock_arduino_output_generator(
        get_target=latest_target_temp
    )

    # Patch the serial connection
    patcher = patch("serial_asyncio.open_serial_connection", new=AsyncMock())
    mock_open: AsyncMock = patcher.start()
    mock_open.return_value = (mock_reader, mock_writer)

    return {
        "reader": mock_reader,
        "writer": mock_writer,
        "open_connection": mock_open,
        "written_data": written_data,
        "patcher": patcher,
    }
