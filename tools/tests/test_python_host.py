import unittest
import json
import struct
import io
from unittest.mock import patch, MagicMock
import os
import sys

# Adjust path to import the script from tools directory
sys.path.append(os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))
import xdm_messaging_host as xdm_host

class TestXdmMessagingHost(unittest.TestCase):

    def test_send_to_browser(self):
        out = io.BytesIO()
        xdm_host.send_to_browser({"test": "data"}, stream=out)

        result = out.getvalue()
        length = struct.unpack('I', result[:4])[0]
        msg = json.loads(result[4:].decode('utf-8'))

        self.assertEqual(msg, {"test": "data"})
        self.assertEqual(length, len(result) - 4)

    def test_read_from_browser(self):
        msg = {"test": "input"}
        msg_json = json.dumps(msg).encode('utf-8')
        input_data = struct.pack('I', len(msg_json)) + msg_json

        inp = io.BytesIO(input_data)
        result = xdm_host.read_from_browser(stream=inp)
        self.assertEqual(result, msg)

    @patch('socket.socket')
    def test_is_xdm_running(self, mock_socket):
        mock_s = MagicMock()
        mock_socket.return_value.__enter__.return_value = mock_s

        mock_s.connect_ex.return_value = 0
        self.assertTrue(xdm_host.is_xdm_running())

        mock_s.connect_ex.return_value = 1
        self.assertFalse(xdm_host.is_xdm_running())

if __name__ == '__main__':
    unittest.main()
