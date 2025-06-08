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

    # Configure readline to return different responses
    mock_reader.readline.side_effect = mock_arduino_output_generator()

    # Patch the serial connection
    patcher = patch("serial_asyncio.open_serial_connection", new=AsyncMock())
    mock_open = patcher.start()
    mock_open.return_value = (mock_reader, mock_writer)

    yield {
        "reader": mock_reader,
        "writer": mock_writer,
        "open_connection": mock_open,
        # "expected_responses": json_responses,
    }

    patcher.stop()
