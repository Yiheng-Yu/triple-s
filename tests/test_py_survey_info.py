import pytest

from triple_s import SurveyInfo


@pytest.mark.parametrize("version", ["1.0", "1.1", "1.2", "2.0", "3.0"])
def test_version_accepts_string_and_float(version):
    from_string = SurveyInfo(version)
    from_float = SurveyInfo(float(version))
    assert repr(from_float.version) == repr(from_string.version)
    assert repr(from_string.version) == f"Version.V{version.replace('.', '_')}"


@pytest.mark.parametrize("version", ["invalid", "2", 1.25, 2.01, float("nan"), float("inf")])
def test_unsupported_version(version):
    with pytest.raises(ValueError):
        SurveyInfo(version)


@pytest.mark.parametrize("version", [None, 2, True, b"2.0", []])
def test_invalid_version_type(version):
    with pytest.raises(TypeError, match="version must be a str or float"):
        SurveyInfo(version)
