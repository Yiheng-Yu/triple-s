import warnings

import pytest
from triple_s import (
    read_data,
    Datamap,
    Question,
    Survey,
    SurveyData,
    Text,
    read_datamap,
    read_triple_s,
)

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"

PATH_SSS_EXAMPLE_2 = "tests/sss-examples/sss-2.0/example2.sss"
PATH_CSV = "tests/sss-examples/sss-2.0/example2.csv"


@pytest.fixture
def datamap():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return read_datamap(PATH_SSS_EXAMPLE_1)


@pytest.mark.parametrize(
    "path_sss, path_data",
    [
        (PATH_SSS_EXAMPLE_1, PATH_ASC),
        (PATH_SSS_EXAMPLE_2, PATH_CSV),
    ],
)
def test_read_data(path_sss, path_data):
    survey = read_triple_s(path_sss, path_data)
    assert isinstance(survey, Survey)
    assert isinstance(survey.datamap, Datamap)
    assert survey.title == "Historic House Exit Survey<br/>First Wave"
    assert survey.has_data()
    assert survey.data.to_polars().shape == (3, 12)


def test_load_asc_file(datamap):
    href = datamap.record.href
    data = read_data(datamap, PATH_ASC)
    assert isinstance(data, SurveyData)
    assert data.has_data()
    assert data.num_response == 3
    assert data.to_polars().shape == (3, 12)
    assert datamap.record.href == href


def test_datamap_iter(datamap):
    question_names = datamap.record.question_names
    record = datamap.record

    for index, question in enumerate(datamap.record):
        correct = question_names[index]
        assert question.name == correct
        assert record[index].name == correct  # check if indexing was correct for Record
        assert (
            datamap[index].name == correct
        )  # check if indexing was correct for Datamap
        assert (
            record.get(correct.lower()).name == record[correct.lower()].name == correct
        )
        assert (
            datamap.get(correct.lower()).name
            == datamap[correct.lower()].name
            == correct
        )

    for container in [datamap, record]:
        for name in ["missing", ""]:
            assert container.get(name) is None
            with pytest.raises(KeyError):
                container[name]


def test_get_empty_datamap():
    datamap = Datamap.from_dict({})
    for container in [datamap, datamap.record]:
        assert container.question_names == []
        assert container.num_questions == 0
        assert container.get("Q1") is None
        assert container.get("") is None


def test_question_response_labels():
    labels = [
        dict(value=1, label="item 1"),
        dict(value=2, label="item 2"),
        dict(value=3, label="item 3"),
        dict(value=4, label="item 4"),
        dict(value=5, label="item 5"),
    ]
    question = Question.from_dict(
        dict(
            name="test",
            label="dummy question label title text",
            question_type="single",
            response=dict(values=labels),
        )
    )

    response_label = question.response_label
    assert response_label[1] == Text("item 1")
    assert response_label[2] == Text("item 2")
    assert response_label[3] == Text("item 3")
    assert response_label[4] == Text("item 4")
    assert response_label[5] == Text("item 5")
