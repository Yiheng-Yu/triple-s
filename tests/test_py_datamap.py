import warnings

import pytest
from triple_s import Datamap, Survey, SurveyData, read_datamap, read_triple_s

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"

PATH_SSS_EXAMPLE_2 = "tests/sss-examples/sss-2.0/example2.sss"
PATH_CSV = "tests/sss-examples/sss-2.0/example2.csv"


@pytest.fixture
def datamap():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return read_datamap(PATH_SSS_EXAMPLE_1)


@pytest.mark.parametrize("path_sss, path_data", [
    (PATH_SSS_EXAMPLE_1, PATH_ASC),
    (PATH_SSS_EXAMPLE_2, PATH_CSV),
])
def test_read_data(path_sss, path_data):
    survey = read_triple_s(path_sss, path_data)
    assert isinstance(survey, Survey)
    assert isinstance(survey.datamap, Datamap)
    assert survey.title == "Historic House Exit Survey<br/>First Wave"
    assert survey.has_data()
    assert survey.data.to_polars().shape == (3, 12)


def test_load_asc_file(datamap):
    data = datamap.load_data(PATH_ASC)
    assert isinstance(data, SurveyData)
    assert data.has_data()
    assert data.num_response == 3
    assert data.to_polars().shape == (3, 12)
    assert datamap.record.href == PATH_ASC


def test_datamap_iter(datamap):
    question_names = datamap.record.question_names
    record = datamap.record
    
    for (index, question) in enumerate(datamap.record):
        correct = question_names[index]
        assert question.name == correct
        assert record[index].name == correct  # check if indexing was correct for Record