import triple_s
from triple_s import DataRange, Text, Variant, Spread, Location, Question, Code, SurveyInfo, MissingKeyError
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


@pytest.mark.parametrize(
    "dic, correct",
    [
        pytest.param(dict(start=1, end=10), DataRange.signed(1, 10), id="integer-range"),
        pytest.param(dict(start=-5, end=10), DataRange.signed(-5, 10), id="signed-range"),
        pytest.param(dict(start=1.5, end=10.5), DataRange.float(1.5, 10.5), id="float-range"),
    ],
)
def test_data_range_from_dict_success(dic, correct):
    assert DataRange.from_dict(dic) == correct


@pytest.mark.parametrize(
    "dic, error",
    [
        pytest.param({"end": 10}, MissingKeyError, id="missing-start"),
        pytest.param({"start": 1}, MissingKeyError, id="missing-end"),
        pytest.param(dict(start=None, end=10), TypeError, id="invalid-start-type"),
        pytest.param(dict(start=1, end=None), TypeError, id="invalid-end-type"),
        pytest.param(dict(start=10, end=1), ValueError, id="end-before-start"),
        pytest.param("random text", TypeError, id="not-a-dictionary"),
    ],
)
def test_data_range_from_dict_fail(dic, error):
    with pytest.raises(error):
        DataRange.from_dict(dic)


@pytest.mark.parametrize(
    "dic, correct",
    [
        pytest.param(dict(code=1), Code.signed(1), id="unsigned-code"),
        pytest.param(dict(code=-1), Code.signed(-1), id="signed-code"),
        pytest.param(dict(code=1.5), Code.float(1.5), id="float-code"),
        pytest.param(dict(code="A"), Code.string("A"), id="string-code"),
        pytest.param(
            dict(code=1, label="Yes", filter="Q1", score=2.5),
            Code.signed(1, "Yes", filter="Q1", score=2.5),
            id="all-fields",
        ),
        pytest.param(
            dict(code=1, label=None, filter=None, score=None),
            Code.signed(1),
            id="null-optional-fields",
        ),
    ],
)
def test_code_from_dict_success(dic, correct):
    instance = Code.from_dict(dic)
    assert instance == correct


@pytest.mark.parametrize(
    "dic, error",
    [
        pytest.param({}, MissingKeyError, id="missing-code"),
        pytest.param(dict(code=None), TypeError, id="null-code"),
        pytest.param(dict(code=[]), TypeError, id="invalid-code-type"),
        pytest.param(dict(code=1, filter=1), TypeError, id="invalid-filter-type"),
        pytest.param(dict(code=1, score="high"), TypeError, id="invalid-score-type"),
        pytest.param("random text", TypeError, id="not-a-dictionary"),
    ],
)
def test_code_from_dict_fail(dic, error):
    with pytest.raises(error):
        Code.from_dict(dic)


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


@pytest.mark.parametrize(
    "dic, expected_type, expected_format, expected_usage",
    [
        pytest.param(
            {"name": "Q1", "question_type": "single"},
            "single", None, None, id="required-fields-and-defaults",
        ),
        pytest.param(
            {
                "name": "Q2",
                "question_type": triple_s.QuestionType.Single,
                "index": 4,
                "format": triple_s.CodeFormat.Literal,
                "usage": triple_s.Usage.Weight,
                "filter": "Q1 > 0",
                "label": {"value": "Title"},
                "location": {"start": 2, "finish": 5},
                "spread": {"num_subfields": 2, "subfield_width": 3},
            },
            "single", "literal", "weight", id="all-fields-from-dictionaries",
        ),
        pytest.param(
            {
                "name": "Q3",
                "question_type": "single",
                "format": None,
                "usage": None,
                "filter": None,
                "label": Text("Existing text"),
                "location": Location(3, 4),
                "spread": Spread(2),
            },
            "single", None, None, id="optional-fields-as-instances-and-none",
        ),
    ],
)
def test_question_from_dict_success(dic, expected_type, expected_format, expected_usage):
    question = Question.from_dict(dic)

    assert question.name == dic["name"]
    assert question.type == expected_type
    assert question.format == expected_format
    assert question.usage == expected_usage

    if dic.get("filter") is not None:
        assert question.filter == dic["filter"]
    else:
        assert question.filter is None

    if dic.get("label") is not None:
        expected_label = "Title" if isinstance(dic["label"], dict) else "Existing text"
        assert question.label.value == expected_label
    else:
        assert question.label.value == ""

    if isinstance(dic.get("location"), dict):
        assert question.location == Location(2, 5)
    elif isinstance(dic.get("location"), Location):
        assert question.location == dic["location"]
    else:
        assert question.location == Location(1)

    if isinstance(dic.get("spread"), dict):
        assert question.spread == Spread(2, 3)
    elif isinstance(dic.get("spread"), Spread):
        assert question.spread == dic["spread"]
    else:
        assert question.spread is None


@pytest.mark.parametrize(
    "dic",
    [
        pytest.param({}, id="missing-name-and-type"),
        pytest.param({"name": "Q1"}, id="missing-question-type"),
        pytest.param({"question_type": "single"}, id="missing-name"),
        pytest.param({"name": 1, "question_type": "single"}, id="invalid-name-type"),
        pytest.param({"name": "Q1", "question_type": "unknown"}, id="invalid-question-type"),
        pytest.param({"name": "Q1", "question_type": "single", "index": -1}, id="invalid-index"),
        pytest.param({"name": "Q1", "question_type": "single", "format": "unknown"}, id="invalid-format"),
        pytest.param({"name": "Q1", "question_type": "single", "usage": "unknown"}, id="invalid-usage"),
        pytest.param({"name": "Q1", "question_type": "single", "filter": 1}, id="invalid-filter-type"),
        pytest.param({"name": "Q1", "question_type": "single", "label": 1}, id="invalid-label-type"),
        pytest.param({"name": "Q1", "question_type": "single", "location": 1}, id="invalid-location-type"),
        pytest.param({"name": "Q1", "question_type": "single", "location": {}}, id="invalid-location-dictionary"),
        pytest.param({"name": "Q1", "question_type": "single", "spread": 1}, id="invalid-spread-type"),
        pytest.param({"name": "Q1", "question_type": "single", "spread": {}}, id="invalid-spread-dictionary"),
        pytest.param({"name": "Q1", "question_type": "single", "response": 1}, id="invalid-response-type"),
        pytest.param("random text", id="not-a-dictionary"),
    ],
)
def test_question_from_dict_fail(dic):
    with pytest.raises(Exception):
        Question.from_dict(dic)


def test_question_from_dict_rejects_response_for_character_question():
    with pytest.raises(ValueError, match="Cannot set 'response'"):
        Question.from_dict(
            {
                "name": "Q1",
                "question_type": "character",
                "response": triple_s.Response(),
            }
        )


@pytest.mark.parametrize(
    "dic, expected_version",
    [
        pytest.param({"version": "1.2"}, triple_s.Version.V1_2, id="version-string-defaults"),
        pytest.param({"version": 2.0}, triple_s.Version.V2_0, id="version-float"),
        pytest.param({"version": triple_s.Version.V3_0}, triple_s.Version.V3_0, id="version-enum"),
    ],
)
def test_survey_info_from_dict_defaults(dic, expected_version):
    info = SurveyInfo.from_dict(dic)

    assert info.version == expected_version
    assert info.name is None
    assert info.title is None
    assert info.date is None
    assert info.time is None
    assert info.origin is None
    assert info.user is None
    assert info.style is None
    assert info.standard is None
    assert info.language_options == ()
    assert info.mode_options == ()


def test_survey_info_from_dict_all_fields():
    info = SurveyInfo.from_dict(
        {
            "version": "2.0",
            "name": "Datamap name",
            "title": "Datamap title",
            "date": "2026-10-23",
            "time": "12:30",
            "origin": "Exporter",
            "user": "Analyst",
            "style": "body { color: black; }",
            "standard": "standardnames",
            "language_options": ["en-GB", "fr"],
            "mode_options": [triple_s.Mode.Interview, "analysis"],
        }
    )

    assert info.version == triple_s.Version.V2_0
    assert info.name == "Datamap name"
    assert info.title == "Datamap title"
    assert info.date == "2026-10-23"
    assert info.time == "12:30"
    assert info.origin == "Exporter"
    assert info.user == "Analyst"
    assert info.style == "body { color: black; }"
    assert info.standard == "standardnames"
    assert info.language_options == ("en-GB", "fr")
    assert info.mode_options == (triple_s.Mode.Interview, triple_s.Mode.Analysis)


def test_survey_info_from_dict_accepts_legacy_language_key_and_nulls():
    info = SurveyInfo.from_dict(
        {
            "version": "1.2",
            "name": None,
            "title": None,
            "language": ["en"],
            "mode_options": None,
        }
    )

    assert info.name is None
    assert info.title is None
    assert info.language_options == ("en",)
    assert info.mode_options == ()


@pytest.mark.parametrize(
    "dic, error",
    [
        pytest.param({}, MissingKeyError, id="missing-version"),
        pytest.param({"version": "4.0"}, ValueError, id="unsupported-version"),
        pytest.param({"version": None}, TypeError, id="null-version"),
        pytest.param({"version": "2.0", "name": 1}, TypeError, id="invalid-name-type"),
        pytest.param({"version": "2.0", "language_options": "en"}, TypeError, id="invalid-language-options-type"),
        pytest.param({"version": "2.0", "language_options": ["not a tag"]}, ValueError, id="invalid-language-tag"),
        pytest.param({"version": "2.0", "mode_options": ["invalid"]}, ValueError, id="invalid-mode-option"),
        pytest.param("random text", TypeError, id="not-a-dictionary"),
    ],
)
def test_survey_info_from_dict_fail(dic, error):
    with pytest.raises(error):
        SurveyInfo.from_dict(dic)
