from copy import copy, deepcopy

import pytest

from triple_s import (
    Code, DataRange, Location, Mode, Question, QuestionType, Response, Spread,
    SurveyInfo, Text, Usage, Variant, Version, read_datamap,
)


@pytest.mark.parametrize("enum, expected", [
    (Version, ["1.0", "1.1", "1.2", "2.0", "3.0"]),
    (QuestionType, ["", "single", "multiple", "logical", "quantity", "character",
                    "date", "time"]),
    (Usage, ["", "serial", "weight"]),
])
def test_enum_available_options(enum, expected):
    assert enum.available_options() == expected
    options = enum.available_options()
    options.clear()
    assert enum.available_options() == expected


@pytest.mark.parametrize("version, expected", [("1.0", Version.V1_0),
                                              (2.0, Version.V2_0),
                                              (Version.V3_0, Version.V3_0)])
def test_survey_info_version_setter(version, expected):
    info = SurveyInfo("1.2")
    info.version = version
    assert info.version == expected


@pytest.mark.parametrize("version, error", [("invalid", ValueError), (9.9, ValueError),
                                          (None, TypeError), ([], TypeError)])
def test_survey_info_version_setter_rejects_invalid_values(version, error):
    info = SurveyInfo("1.2")
    with pytest.raises(error):
        info.version = version
    assert info.version == Version.V1_2


def test_survey_info_to_dict_roundtrip_and_independence():
    fields = {
        "name": "Survey", "title": "Title", "date": "2026-01-01", "time": "12:30",
        "origin": "Exporter", "user": "Analyst", "style": "style",
        "standard": "standardnames",
    }
    info = SurveyInfo("2.0", **fields, language_options=["en", "fr"],
                      mode_options=["interview", "analysis"])
    result = info.to_dict()
    assert result == {
        **fields, "version": Version.V2_0, "language": ["en", "fr"],
        "mode_options": [Mode.Interview, Mode.Analysis],
    }
    restored = SurveyInfo.from_dict(result)
    assert restored.to_dict() == result
    result["language"].append("de")
    result["mode_options"].clear()
    result["title"] = "Changed"
    assert info.language_options == restored.language_options == ("en", "fr")
    assert info.mode_options == (Mode.Interview, Mode.Analysis)
    assert info.title == restored.title == "Title"


def test_survey_info_to_dict_omits_missing_optional_fields():
    assert SurveyInfo("1.2").to_dict() == {
        "version": Version.V1_2, "language": [], "mode_options": [],
    }


def test_spread_properties_and_shared_question_view():
    spread = Spread(2)
    assert spread.num_subfields == 2
    assert spread.subfield_width is None
    question = Question("Q1", "multiple", spread=spread)
    question.spread.num_subfields = 3
    question.spread.subfield_width = 2
    assert spread == Spread(3, 2)
    spread.subfield_width = None
    assert question.spread.subfield_width is None


@pytest.mark.parametrize("field, value, error", [
    ("num_subfields", 0, ValueError), ("num_subfields", -1, OverflowError),
    ("num_subfields", None, TypeError), ("num_subfields", 1.5, TypeError),
    ("subfield_width", 0, ValueError), ("subfield_width", -1, OverflowError),
    ("subfield_width", "2", TypeError),
])
def test_spread_setters_reject_invalid_values(field, value, error):
    spread = Spread(2, 1)
    with pytest.raises(error):
        setattr(spread, field, value)
    assert spread == Spread(2, 1)


def test_location_defaults_and_setters():
    location = Location(2)
    assert location.start == location.finish == 2
    location.finish = 5
    location.start = 3
    assert location == Location(3, 5)
    with pytest.raises(ValueError):
        location.finish = 2
    assert location == Location(3, 5)
    with pytest.raises(ValueError):
        Location(5, 3)


@pytest.mark.parametrize("clone", [copy, deepcopy])
def test_text_copy_preserves_independent_variants(clone):
    original = Text("Original", [Variant("English", language="en")])
    copied = clone(original)
    assert copied == original
    copied.value = "Changed"
    copied.variants[0].value = "Changed variant"
    copied.add_variant("French", language="fr")
    assert original.value == "Original"
    assert original.variants == (Variant("English", language="en"),)


@pytest.mark.parametrize("clone", [Question.copy, copy, deepcopy])
def test_question_copy_preserves_independent_nested_fields(clone):
    original = Question("Q1", "multiple", label="Original", location=Location(1, 4),
                        spread=Spread(2, 2), filter="Q0")
    original.label.add_variant("English", language="en")
    original.response = Response(DataRange.unsigned(1, 9), [Code.unsigned(1, "One")])
    copied = clone(original)
    assert copied.to_xml() == original.to_xml()
    copied.name = "Q2"
    copied.label.value = "Changed"
    copied.label.variants[0].value = "Changed variant"
    copied.location.finish = 6
    copied.spread.num_subfields = 3
    copied.filter = "Q9"
    copied.response.range.end = 10
    copied.response[0].label.value = "Changed code"
    copied.response.add_value(Code.unsigned(2, "Two"))
    assert original.name == "Q1"
    assert original.label.value == "Original"
    assert original.label.variants[0].value == "English"
    assert original.location == Location(1, 4)
    assert original.spread == Spread(2, 2)
    assert original.filter == "Q0"
    assert original.response.range == DataRange.unsigned(1, 9)
    assert original.response[0].label.value == "One"
    assert len(original.response.values) == 1


def test_question_classic_export_roundtrip():
    datamap = read_datamap("tests/sss-examples/sss_classic.sss")
    for original in datamap:
        restored = Question.from_string(original.to_classic())
        assert restored.to_xml() == original.to_xml()


def test_datamap_classic_export_roundtrip(tmp_path):
    original = read_datamap("tests/sss-examples/sss_classic.sss")
    path = tmp_path / "export.sss"
    path.write_text(original.to_classic(), encoding="utf-8")
    restored = read_datamap(str(path))
    assert restored.question_names == original.question_names
    assert restored.info.to_dict() == original.info.to_dict()
    assert restored.record.to_xml() == original.record.to_xml()


@pytest.mark.parametrize("question_type", [None, "date", "time"])
def test_question_classic_export_rejects_unsupported_types(question_type):
    with pytest.raises(ValueError):
        Question("Q1", question_type, location=Location(1)).to_classic()


def test_question_classic_export_rejects_literal_codes():
    question = Question("Q1", "single", format="literal", location=Location(1))
    question.response = Response(values=[Code.string("A", "Option A")])
    with pytest.raises(ValueError):
        question.to_classic()


@pytest.mark.parametrize("fragment", ["", "not a question", "<variable>"])
def test_question_from_string_rejects_invalid_fragments(fragment):
    with pytest.raises(ValueError):
        Question.from_string(fragment)
