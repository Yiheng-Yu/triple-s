import triple_s
from triple_s import Text, Variant, Spread, Location, Question, MissingKeyError
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
        pytest.param(
            dict(value="test", mode=triple_s.Mode.Analysis, language=None),
            Variant("test", "analysis", None),
            id="analysis-mode-with-language-none",
        ),
        pytest.param(
            dict(value="test", mode=None, language="en-GB"),
            Variant("test", None, "en-GB"),
            id="language-with-null-mode",
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
    "dic, error",
    [
        pytest.param({"mode": "interview"}, MissingKeyError, id="missing-value"),
        pytest.param({"value": "test"}, MissingKeyError, id="missing-mode"),
        pytest.param(
            {"value": "test", "mode": None, "language": None},
            MissingKeyError,
            id="both-options-none",
        ),
        pytest.param({"value": "3", "mode": "ok"}, ValueError, id="invalid-mode"),
        pytest.param({"value": "3", "language": "what"}, ValueError, id="invalid-language"),
        pytest.param("random text", TypeError, id="type-err"),
    ],
)
def test_variant_from_dict_fail(dic, error):
    with pytest.raises(error):
        Variant.from_dict(dic)


@pytest.mark.parametrize(
    "dic, correct",
    [
        pytest.param(dict(value="haha"), Text("haha"), id="no-variant"),
        pytest.param(dict(value="haha", variants=None), Text("haha"), id="null-variants"),
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


def test_text_from_dict_missing_key():
    with pytest.raises(MissingKeyError):
        Text.from_dict(dict(variants=[dict(value="yes", mode="interview"), Variant("haha", language="en-GB")]))


@pytest.mark.parametrize(
    "dic",
    [
        pytest.param("random text", id="type-err"),
    ],
)
def test_text_from_dict_err(dic):
    with pytest.raises(TypeError):
        Text.from_dict(dic)


@pytest.mark.parametrize(
    "dic, correct",
    [
        pytest.param(dict(num_subfields=3), Spread(3), id="no-width"),
        pytest.param(dict(num_subfields=3, subfield_width=None), Spread(3), id="null-width"),
        pytest.param(dict(num_subfields=3, subfield_width=2), Spread(3, 2), id="with-width"),
    ],
)
def test_spread_from_dict_success(dic, correct):
    assert Spread.from_dict(dic) == correct


@pytest.mark.parametrize(
    "dic",
    [
        pytest.param({"mode": "interview"}, id="missing-num_subfields"),
        pytest.param({"num_subfields": "interview"}, id="type-err-num_subfields"),
        pytest.param("random text", id="type-err"),
    ],
)
def test_spread_from_dict_fail(dic):
    if isinstance(dic, str):
        error = TypeError
    elif "num_subfields" not in dic:
        error = MissingKeyError
    else:
        error = TypeError
    with pytest.raises(error):
        Spread.from_dict(dic)


@pytest.mark.parametrize(
    "dic, correct",
    [
        pytest.param(dict(start=1), Location(1), id="start-only"),
        pytest.param(dict(start=1, finish=None), Location(1), id="null-finish"),
        pytest.param(dict(start=1, finish=20), Location(1, 20), id="start-and-finish"),
    ],
)
def test_location_from_dict_success(dic, correct):
    assert Location.from_dict(dic) == correct


@pytest.mark.parametrize(
    "dic, error",
    [
        pytest.param({}, MissingKeyError, id="missing-start"),
        pytest.param(dict(start="one"), TypeError, id="invalid-start-type"),
        pytest.param(dict(start=2, finish=1), ValueError, id="finish-before-start"),
        pytest.param("random text", TypeError, id="not-a-dictionary"),
    ],
)
def test_location_from_dict_fail(dic, error):
    with pytest.raises(error):
        Location.from_dict(dic)


def test_question_from_dict_null_optional_keys():
    question = Question.from_dict(
        {
            "name": "Q1",
            "question_type": triple_s.QuestionType.Single,
            "format": None,
            "label": None,
            "location": None,
            "spread": None,
        }
    )

    assert question.name == "Q1"
    assert question.type == "single"
    assert question.format is None
    assert question.label.value == ""
    assert question.location == Location(1)
    assert question.spread is None
