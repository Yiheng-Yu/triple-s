"""Install the matching published wheel for Read the Docs without compiling."""

import subprocess
import sys
import tomllib
from pathlib import Path


def main():
    root = Path(__file__).resolve().parents[1]
    with (root / "Cargo.toml").open("rb") as manifest:
        version = tomllib.load(manifest)["package"]["version"]
    requirement = f"triple-s-survey=={version}"
    print(
        f"Installing {requirement} from a published wheel. "
        "Publish this version's Linux wheel before rebuilding these docs. "
        "Source builds are disabled, including for dependencies.",
        flush=True,
    )
    subprocess.run(
        [
            sys.executable, "-m", "pip", "install", "--verbose",
            "--only-binary=:all:", requirement,
        ],
        check=True,
    )


if __name__ == "__main__":
    main()
