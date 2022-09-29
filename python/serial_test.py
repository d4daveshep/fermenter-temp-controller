# from mock_serial import MockSerial
from serial import Serial
# import pytest

import logging, sys

logging.basicConfig(
  stream=sys.stdout,
  level=logging.DEBUG,
  format="%(levelname)s - %(message)s"
)

def test_example(mock_serial):

    serial = Serial(mock_serial.port)  # open a mocked serial port

    # create a stub ojbect that we can use multiple timees,  the
    stub = mock_serial.stub(
        receive_bytes=b'123',
        send_bytes=b'456'
    )

    serial.write(b'123')
    assert serial.read(3) == b'456'
    serial.write(b'123')
    assert serial.read(3) == b'456'

    assert stub.called
    assert stub.calls == 1

    serial.close()

