"""Install the published package for autodoc without compiling."""

import json
import subprocess
import sys
import tomllib
from pathlib import Path
from urllib.error import HTTPError
from urllib.request import urlopen


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

    if not any(file["packagetype"] == "bdist_wheel" and not file.get("yanked") for file in files):
        print(f"Wheel for {version} not published yet; cancelling this docs build.")
        return 183  # Read the Docs cancellation code.

    subprocess.run(
        [sys.executable, "-m", "pip", "install", "--verbose",
         "--only-binary=:all:", f"triple-s-survey=={version}"],
        check=True,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
