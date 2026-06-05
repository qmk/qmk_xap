import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parent))

import sync_keycodes


class SyncKeycodesTest(unittest.TestCase):
    def test_resolve_qmk_firmware_requires_argument_or_envvar(self):
        with patch.dict(os.environ, {}, clear=True):
            with self.assertRaisesRegex(sync_keycodes.SyncError, "Pass --qmk-firmware"):
                sync_keycodes._resolve_qmk_firmware(None)

    def test_sync_uses_qmk_loader_and_writes_standalone_version_files(self):
        with tempfile.TemporaryDirectory() as qmk_dir, tempfile.TemporaryDirectory() as output_dir:
            qmk_firmware = Path(qmk_dir)
            qmk_package = qmk_firmware / "lib/python/qmk"
            qmk_package.mkdir(parents=True)
            (qmk_package / "__init__.py").write_text("", encoding="utf-8")
            (qmk_package / "keycodes.py").write_text(
                """
from pathlib import Path


def list_versions():
    return ["0.0.2", "0.0.1"]


def load_spec(version):
    with Path("loader_called.txt").open("a", encoding="utf-8") as out_file:
        out_file.write(version + "\\n")
    return {
        "version": version,
        "ranges": {
            "0x7E00/0x003F": {"define": "QK_KB_" + version.replace(".", "_")}
        },
        "keycodes": {
            "0x0002": {"key": "KC_B_" + version.replace(".", "_")},
            "0x0001": {"key": "KC_A_" + version.replace(".", "_")},
        },
    }
""",
                encoding="utf-8",
            )

            output = Path(output_dir)
            stale_output = output / "keycodes_0.0.0.generated.hjson"
            stale_output.write_text("stale", encoding="utf-8")
            sys.modules.pop("qmk", None)
            sys.modules.pop("qmk.keycodes", None)

            outputs = sync_keycodes.sync_keycodes(qmk_firmware, output)

            self.assertEqual(
                (qmk_firmware / "loader_called.txt").read_text(encoding="utf-8"),
                "0.0.1\n0.0.2\n",
            )
            self.assertEqual(
                outputs,
                [
                    output / "keycodes_0.0.1.generated.hjson",
                    output / "keycodes_0.0.2.generated.hjson",
                ],
            )
            self.assertFalse(stale_output.exists())
            self.assertEqual(
                json.loads((output / "keycodes_0.0.1.generated.hjson").read_text(encoding="utf-8")),
                {
                    "version": "0.0.1",
                    "ranges": {
                        "0x7E00/0x003F": {"define": "QK_KB_0_0_1"},
                    },
                    "keycodes": {
                        "0x0001": {"key": "KC_A_0_0_1"},
                        "0x0002": {"key": "KC_B_0_0_1"},
                    },
                },
            )
            self.assertEqual(
                json.loads((output / "keycodes_0.0.2.generated.hjson").read_text(encoding="utf-8")),
                {
                    "version": "0.0.2",
                    "ranges": {
                        "0x7E00/0x003F": {"define": "QK_KB_0_0_2"},
                    },
                    "keycodes": {
                        "0x0001": {"key": "KC_A_0_0_2"},
                        "0x0002": {"key": "KC_B_0_0_2"},
                    },
                },
            )


if __name__ == "__main__":
    unittest.main()
