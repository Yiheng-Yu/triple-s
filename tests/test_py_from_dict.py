import triple_s
from triple_s import Text, Variant, Mode, MissingKeyError
import pytest


@pytest.mark.parametrize(
    "dic, correct",
    [
        pytest.param(
            dict(value="test", mode="interview"),
            Variant("test", "interview", None),
            id="interview-mode",
        ),
        pytest.param(
            dict(value="test", mode="analysis"),
            Variant("test", "analysis", None),
            id="analysis-mode",
        ),
        pytest.param(
            dict(value="test", language="en-GB"),
            Variant("test", None, "en-GB"),
            id="language-only",
        ),
        pytest.param(
            dict(value="test", mode=triple_s.Mode.Interview, language="en-GB"),
            Variant("test", "interview", "en-GB"),
            id="interview-mode-with-language",
        ),
        pytest.param(
            dict(value="test", mode=triple_s.Mode.Analysis, language="en-GB"),
            Variant("test", "analysis", "en-GB"),
            id="analysis-mode-with-language",
        ),
    ],
)
def test_variant_from_dict_success(dic, correct):
    new = Variant.from_dict(dic)
    assert new == correct


@pytest.mark.parametrize("dic", [dict(mode="interview"), dict(value="test")])
def test_variant_from_dict_missing_key_err(dic):
    with pytest.raises(MissingKeyError):
        Variant.from_dict(dic)


@pytest.mark.parametrize(
    "dic",
    [
        pytest.param({"mode": "interview"}, id="missing-value"),
        pytest.param({"value": "test"}, id="missing-mode"),
        pytest.param({"value": "3", "mode": "ok"}, id="invalid-mode"),
        pytest.param({"value": "3", "language": "what"}, id="invalid-language"),
    ],
)
def test_variant_from_dict_fail(dic):
    with pytest.raises(Exception):
        Variant.from_dict(dic)


@pytest.mark.parametrize(
    "dic, correct",
    [
        pytest.param(dict(value="haha"), Text("haha"), id="no-variant"),
        pytest.param(
            dict(value="haha", variants=[dict(value="yes", mode="interview")]),
            Text("haha", variants=[Variant("yes", mode="interview")]),
            id="has-one-variant",
        ),
        pytest.param(
            dict(value="haha", variants=[dict(value="yes", mode="interview"), Variant("haha", language="en-GB")]),
            Text("haha", variants=[Variant("haha", language="en-GB"), Variant("yes", mode="interview")]),
            id="has-multiple-variants",
        ),
    ],
)
def test_text_from_dict_success(dic, correct):
    assert Text.from_dict(dic) == correct
