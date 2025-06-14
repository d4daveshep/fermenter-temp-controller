from typing import Any
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from mock_arduino_controller.mock_temperature import mock_arduino_output_generator


@pytest.fixture
def mock_serial_connection():
    """Fixture to create a mock serial connection for read/write operations."""
    mock_reader = AsyncMock()
    mock_writer = MagicMock()

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
    mock_open = patcher.start()
    mock_open.return_value = (mock_reader, mock_writer)

    yield {
        "reader": mock_reader,
        "writer": mock_writer,
        "open_connection": mock_open,
        "written_data": written_data,
    }

    patcher.stop()
