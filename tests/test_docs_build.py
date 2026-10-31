"""Check the publication gate and wheel-only installation."""

import io
import json
import runpy
from pathlib import Path
from unittest.mock import patch
from urllib.error import HTTPError

import pytest
from packaging.tags import Tag

SCRIPT = Path(__file__).resolve().parents[1] / "docs/install_package.py"
WHEEL = {
    "packagetype": "bdist_wheel",
    "filename": "triple_s_survey-0.4.12-cp310-abi3-manylinux_2_28_x86_64.whl",
    "url": "https://files.pythonhosted.org/packages/test/triple_s_survey-0.4.12-cp310-abi3-manylinux_2_28_x86_64.whl",
}


@pytest.mark.parametrize("files, expected", [
    ([], 183),
    ([{"packagetype": "sdist"}], 183),
    ([{"packagetype": "bdist_wheel", "yanked": True}], 183),
    ([WHEEL], 0),
])
def test_publication_gate(files, expected):
    response = io.BytesIO(json.dumps({"urls": files}).encode())
    with patch("urllib.request.urlopen", return_value=response), \
         patch("packaging.tags.sys_tags", return_value=[Tag("cp310", "abi3", "manylinux_2_28_x86_64")]), \
         patch("subprocess.run") as install, pytest.raises(SystemExit) as result:
        runpy.run_path(str(SCRIPT), run_name="__main__")
    assert result.value.code == expected
    if expected == 0:
        assert "--only-binary=:all:" in install.call_args.args[0]
        assert install.call_args.args[0][-1] == WHEEL["url"]
        assert install.call_args.kwargs["check"] is True
    else:
        install.assert_not_called()


def test_incompatible_wheel_is_reported_without_installing():
    response = io.BytesIO(json.dumps({"urls": [WHEEL]}).encode())
    with patch("urllib.request.urlopen", return_value=response), \
         patch("packaging.tags.sys_tags", return_value=[Tag("cp310", "abi3", "macosx_11_0_arm64")]), \
         patch("subprocess.run") as install, pytest.raises(SystemExit, match="No wheel supports"):
        runpy.run_path(str(SCRIPT), run_name="__main__")
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
