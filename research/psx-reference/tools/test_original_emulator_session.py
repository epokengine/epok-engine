"""Transport and output-safety checks; no emulator or original assets required."""
import io
from pathlib import Path
import tempfile
import unittest

from original_emulator_session import Frames, child, save_frame
from PIL import Image


class Connection:
    def __init__(self, reply):
        self.reply = io.BytesIO(reply)
        self.sent = []

    def makefile(self, mode):
        return self.reply

    def sendall(self, data):
        self.sent.append(data)


class ObserverTests(unittest.TestCase):
    def test_error_consumed_before_next_frame(self):
        error = b"ERROR: missing optional API"
        connection = Connection(b"CSD1 %d\n" % len(error) + error +
                                b"CSF1 1 1 2 7 0 42 3 30.0 2147483648\n\xff\x7f")
        frames = Frames(connection)
        with self.assertRaisesRegex(RuntimeError, "missing optional API"):
            frames.frame()
        metadata, pixels = frames.frame()
        self.assertEqual(metadata["cycles"], 42)
        self.assertEqual(metadata["guest_fps"], 30)
        self.assertEqual(pixels, b"\xff\x7f")

    def test_truncated_frame(self):
        with self.assertRaisesRegex(ValueError, "payload"):
            Frames(Connection(b"CSF1 1 1 2 1 0 1 1 0 0\n\x00")).frame()

    def test_invalid_payload_size(self):
        for value in (-1, 33554433):
            with self.assertRaisesRegex(ValueError, "size"):
                Frames(Connection(f"CSD1 {value}\n".encode())).data("M")

    def test_output_guards(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            for name in ("../out.png", "out.txt", str(folder / "absolute.png")):
                with self.assertRaises(ValueError):
                    child(folder, name, ".png")
            (folder / "out.json").touch()
            with self.assertRaises(ValueError):
                child(folder, "out.png", ".png")

    def test_rgb555_channels_and_metadata(self):
        with tempfile.TemporaryDirectory() as directory:
            folder = Path(directory)
            metadata = dict(width=3, height=1, depth=2)
            save_frame(folder, "rgb.png", (metadata, b"\x1f\x00\xe0\x03\x00\x7c"))
            with Image.open(folder / "rgb.png") as image:
                self.assertEqual([image.getpixel((i, 0)) for i in range(3)],
                                 [(255, 0, 0), (0, 255, 0), (0, 0, 255)])


if __name__ == "__main__":
    unittest.main()
