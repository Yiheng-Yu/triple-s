"""Install the published package for autodoc without compiling."""

import json
import subprocess
import sys
import tomllib
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import urlopen

from packaging.tags import sys_tags
from packaging.utils import parse_wheel_filename


def main():
    manifest = Path(__file__).resolve().parents[1] / "Cargo.toml"
    version = tomllib.loads(manifest.read_text())["package"]["version"]
    try:
        with urlopen(f"https://pypi.org/pypi/triple-s-survey/{version}/json", timeout=30) as response:
            files = json.load(response)["urls"]
    except HTTPError as error:
        if error.code != 404:
            raise
        files = []

    wheels = [file for file in files if file["packagetype"] == "bdist_wheel" and not file.get("yanked")]
    if not wheels:
        print(f"Wheel for {version} not published yet; cancelling this docs build.")
        return 183  # Read the Docs cancellation code.

    supported = set(sys_tags())
    wheel = next((file for file in wheels if parse_wheel_filename(file["filename"])[3] & supported), None)
    if wheel is None:
        raise SystemExit(f"No wheel supports this Python/platform: {[file['filename'] for file in wheels]}")

    print(f"Installing {wheel['filename']}", flush=True)
    subprocess.run(
        [sys.executable, "-m", "pip", "install", "--verbose",
         "--only-binary=:all:", wheel["url"]],
        check=True,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
