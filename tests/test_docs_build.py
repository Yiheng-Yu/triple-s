"""Check the publication gate and wheel-only installation."""

import io
import json
import runpy
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError

import pytest

SCRIPT = Path(__file__).resolve().parents[1] / "docs/install_package.py"


@pytest.mark.parametrize("files, expected", [
    ([], 183),
    ([{"packagetype": "sdist"}], 183),
    ([{"packagetype": "bdist_wheel", "yanked": True}], 183),
    ([{"packagetype": "bdist_wheel"}], 0),
])
def test_publication_gate(files, expected):
    response = io.BytesIO(json.dumps({"urls": files}).encode())
    with patch("urllib.request.urlopen", return_value=response), \
         patch("subprocess.run") as install, pytest.raises(SystemExit) as result:
        runpy.run_path(str(SCRIPT), run_name="__main__")
    assert result.value.code == expected
    if expected == 0:
        assert "--only-binary=:all:" in install.call_args.args[0]
        assert install.call_args.kwargs["check"] is True
    else:
        install.assert_not_called()


@pytest.mark.parametrize("status", [404, 500])
def test_missing_release_cancels_but_server_errors_fail(status):
    error = HTTPError("url", status, "error", {}, None)
    with patch("urllib.request.urlopen", side_effect=error), \
         patch("subprocess.run") as install:
        with pytest.raises(SystemExit if status == 404 else HTTPError) as result:
            runpy.run_path(str(SCRIPT), run_name="__main__")
        assert result.value.code == (183 if status == 404 else 500)
        install.assert_not_called()
