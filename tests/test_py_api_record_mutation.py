import pytest

from triple_s import Datamap, DataFormat, Location, Question, Record, SurveyInfo, Text


def test_record_empty_and_metadata():
    record = Record.empty()
    assert len(record) == record.num_questions == 0
    assert record.question_names == []
    assert record.content == ()
    assert list(record) == []
    assert record.href is record.skip is record.data_format is None
    record.ident = "B"
    record.href = "answers.csv"
    record.skip = "2"
    assert (record.ident, record.href, record.skip) == ("B", "answers.csv", "2")
    record.href = record.skip = None
    assert record.href is record.skip is None


def test_record_add_question_copies_nested_fields():
    record = Record.empty()
    original = Question("Q1", "single", label="Original", location=Location(1))
    original.label.add_variant("English", language="en")
    assert record.add_question(original) is None
    assert record.num_questions == len(record) == 1
    assert record.question_names == ["Q1"]
    original.label.value = "Changed outside the record"
    original.label.variants[0].value = "Changed variant"
    assert record[0].label.value == "Original"
    assert record[0].label.variants[0].value == "English"
    record[0].location.finish = 3
    assert original.location.finish == 1


def test_record_add_question_replaces_case_insensitively_in_place():
    record = Record.from_dict({"content": [
        {"name": "Q1", "question_type": "single"},
        {"name": "Q2", "question_type": "single"},
    ]})
    record.add_question(Question("q1", "quantity", label="Replacement"))
    assert len(record) == 2
    assert record.question_names == ["q1", "Q2"]
    assert record.get("Q1").type == "quantity"
    assert record[0].label == Text("Replacement")


def test_record_remove_question_returns_independent_copy():
    record = Record.from_dict({"content": [
        {"name": "Q1", "question_type": "single", "label": "Original"},
        {"name": "Q2", "question_type": "single"},
    ]})
    shared = record[0]
    removed = record.remove_question("q1")
    assert removed.name == "Q1"
    assert removed.label.value == "Original"
    assert record.question_names == ["Q2"]
    assert len(record) == record.num_questions == 1
    assert record.get("Q1") is None
    removed.label.value = "Changed removed copy"
    assert shared.label.value == "Original"
    assert record.remove_question("q1") is None
    assert record.remove_question("missing") is None
    assert record.question_names == ["Q2"]


@pytest.mark.parametrize("value, expected", [
    ("fixed", DataFormat.Fixed), ("csv", DataFormat.Csv),
    (DataFormat.Fixed, DataFormat.Fixed), (DataFormat.Csv, DataFormat.Csv),
    (None, None),
])
def test_record_data_format_setter(value, expected):
    record = Record.empty()
    record.data_format = value
    assert record.data_format == expected


@pytest.mark.parametrize("value, error", [("invalid", ValueError), (42, TypeError),
                                         ([], TypeError)])
def test_record_data_format_setter_rejects_invalid_values(value, error):
    record = Record.from_dict({"data_format": "csv"})
    with pytest.raises(error):
        record.data_format = value
    assert record.data_format == DataFormat.Csv


@pytest.mark.parametrize("as_objects", [False, True])
def test_record_from_dict_metadata_and_mixed_questions(as_objects):
    first = Question("Q1", "single")
    content = [first, {"name": "Q2", "question_type": "quantity"}]
    record = Record.from_dict({
        "ident": "Z", "href": "answers.csv", "skip": "1", "content": content,
        "data_format": DataFormat.Csv if as_objects else "csv",
    })
    assert (record.ident, record.href, record.skip) == ("Z", "answers.csv", "1")
    assert record.data_format == DataFormat.Csv
    assert record.question_names == ["Q1", "Q2"]
    assert [item.type for item in record.content] == ["single", "quantity"]
    defaults = Record.from_dict({"ident": None, "href": None, "data_format": None})
    assert defaults.ident == "A"
    assert defaults.href is defaults.data_format is None


@pytest.mark.parametrize("content, error", [
    ({"ident": "AB"}, ValueError), ({"ident": 1}, TypeError),
    ({"href": 1}, TypeError), ({"skip": 1}, TypeError),
    ({"content": [42]}, TypeError), ({"content": None}, TypeError),
    ({"data_format": "invalid"}, ValueError), ({"data_format": 42}, TypeError),
])
def test_record_from_dict_rejects_invalid_fields(content, error):
    with pytest.raises(error):
        Record.from_dict(content)


@pytest.mark.parametrize("info_key", ["info", "survey_info"])
@pytest.mark.parametrize("as_objects", [False, True])
def test_datamap_from_dict_accepts_info_aliases_and_objects(info_key, as_objects):
    info = {"version": "2.0", "name": "Survey", "title": "Title"}
    record = {"ident": "A", "content": [{"name": "Q1", "question_type": "single"}]}
    if as_objects:
        info = SurveyInfo.from_dict(info)
        record = Record.from_dict(record)
    datamap = Datamap.from_dict({info_key: info, "record": record})
    assert datamap.info.name == "Survey"
    assert datamap.title == "Title"
    assert datamap.question_names == ["Q1"]
    assert datamap.num_questions == 1
    datamap.record.add_question(Question("Q2", "single"))
    assert datamap.question_names == ["Q1", "Q2"]
    datamap.record.remove_question("q1")
    assert datamap.question_names == ["Q2"]


@pytest.mark.parametrize("content", [{}, {"info": None, "record": None}])
def test_datamap_from_dict_optional_defaults(content):
    datamap = Datamap.from_dict(content)
    assert datamap.info.name is datamap.title is None
    for container in (datamap, datamap.record):
        assert container.num_questions == 0
        assert container.question_names == []
        assert list(container) == []
        for name in ("Q1", ""):
            assert container.get(name) is None


@pytest.mark.parametrize("key", ["info", "survey_info", "record"])
@pytest.mark.parametrize("value", [42, [], "invalid"])
def test_datamap_from_dict_rejects_invalid_children(key, value):
    with pytest.raises(TypeError):
        Datamap.from_dict({key: value})
