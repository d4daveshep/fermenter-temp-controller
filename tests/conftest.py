from typing import Any, Generator
from unittest.mock import AsyncMock, MagicMock, patch

import pytest

from mock_arduino_controller.mock_serial_connection import patch_serial_connection
from mock_arduino_controller.mock_temperature import mock_arduino_output_generator


@pytest.fixture
def mock_serial_connection() -> Generator[dict[str, Any], None, None]:
    """
    Fixture to create a mock serial connection for read/write operations.
    """

    # Patch the serial connection to use as fixture
    patched: dict[str, Any] = patch_serial_connection()

    # Yield the patched objects as a dictionary
    yield patched

    # Stop the patcher at the end of the test
    patcher = patched["patcher"]
    patcher.stop()
