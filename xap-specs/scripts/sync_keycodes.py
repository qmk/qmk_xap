#!/usr/bin/env python3
"""Generate flattened XAP keycode constants from a QMK firmware checkout."""

from __future__ import annotations

import argparse
import json
import os
import sys
from collections.abc import Mapping
from pathlib import Path
from typing import Any


DEFAULT_OUTPUT_DIR = Path("xap-specs/assets")
GENERATED_KEYCODE_GLOB = "keycodes_*.generated.hjson"
GENERATED_KEYCODE_TEMPLATE = "keycodes_{version}.generated.hjson"


class SyncError(RuntimeError):
    """A user-facing sync failure."""


def _repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def _resolve_qmk_firmware(raw_path: str | None) -> Path:
    path = raw_path or os.environ.get("QMK_FIRMWARE")

    if not path:
        raise SyncError("Pass --qmk-firmware or set QMK_FIRMWARE.")

    qmk_firmware = Path(path).expanduser().resolve()
    keycodes_py = qmk_firmware / "lib/python/qmk/keycodes.py"

    if not keycodes_py.is_file():
        raise SyncError(f"{qmk_firmware} does not look like a QMK firmware checkout.")

    return qmk_firmware


def _load_qmk_keycodes(qmk_firmware: Path) -> dict[str, Any]:
    sys.path.insert(0, str(qmk_firmware / "lib/python"))

    try:
        from qmk.keycodes import list_versions, load_spec
    except ModuleNotFoundError as err:
        missing = err.name or str(err)
        raise SyncError(
            f"Failed to import QMK's keycode loader because Python module {missing!r} is missing. "
            f"Install QMK's Python requirements from {qmk_firmware / 'requirements.txt'}."
        ) from err

    old_cwd = Path.cwd()
    try:
        os.chdir(qmk_firmware)
        versions = list(reversed(list_versions()))
        if not versions:
            raise SyncError(f"No keycode versions found in {qmk_firmware}.")

        return {version: _stable_keycode_spec(load_spec(version)) for version in versions}
    finally:
        os.chdir(old_cwd)


def _stable_keycode_spec(spec: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "version": spec["version"],
        "keycodes": dict(sorted(spec.get("keycodes", {}).items())),
        "ranges": dict(sorted(spec.get("ranges", {}).items())),
    }


def sync_keycodes(qmk_firmware: Path, output_dir: Path) -> list[Path]:
    specs = _load_qmk_keycodes(qmk_firmware)

    output_dir.mkdir(parents=True, exist_ok=True)

    generated_files = {
        output_dir / GENERATED_KEYCODE_TEMPLATE.format(version=version)
        for version in specs
    }

    for old_generated_file in output_dir.glob(GENERATED_KEYCODE_GLOB):
        if old_generated_file not in generated_files:
            old_generated_file.unlink()

    for version, spec in specs.items():
        output = output_dir / GENERATED_KEYCODE_TEMPLATE.format(version=version)
        with output.open("w", encoding="utf-8") as out_file:
            json.dump(spec, out_file, indent=2, sort_keys=True)
            out_file.write("\n")

    return sorted(generated_files)


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--qmk-firmware",
        help="Path to a qmk_firmware checkout. Falls back to QMK_FIRMWARE.",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=DEFAULT_OUTPUT_DIR,
        help=(
            "Output directory relative to the qmk_xap repo root. "
            f"Default: {DEFAULT_OUTPUT_DIR}"
        ),
    )
    return parser.parse_args()


def main() -> int:
    args = _parse_args()

    try:
        qmk_firmware = _resolve_qmk_firmware(args.qmk_firmware)
        output_dir = args.output_dir
        if not output_dir.is_absolute():
            output_dir = _repo_root() / output_dir
        outputs = sync_keycodes(qmk_firmware, output_dir)
    except SyncError as err:
        print(f"error: {err}", file=sys.stderr)
        return 1

    print(f"Synced {len(outputs)} QMK keycode versions from {qmk_firmware} to {output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
