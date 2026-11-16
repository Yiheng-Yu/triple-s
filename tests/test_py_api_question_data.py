import gc

import pandas as pd
import polars as pl
import pytest
from polars.testing import assert_frame_equal, assert_series_equal
from triple_s import read_data, NoDataError, QuestionData, Survey, SurveyData, read_triple_s

PATH_SSS = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS, PATH_ASC)


def test_view_survives_owner():
    survey = read_triple_s(PATH_SSS, PATH_ASC)
    view = survey["Q2"]
    expected = survey.data.to_polars().get_column("Q2")
    assert isinstance(view, QuestionData)
    assert view.name == "Q2"
    assert isinstance(view.to_polars(), pl.Series)
    assert view.name == "Q2"
    del survey
    gc.collect()
    assert_series_equal(view.to_polars(), expected)


def test_view_follows_data_replacement(survey):
    view = survey["Q2"]
    replacement = survey.data.to_polars().reverse()
    survey.data = SurveyData.from_polars(replacement)
    assert_series_equal(view.to_polars(), replacement.get_column("Q2"))
    survey.data = SurveyData.from_polars(replacement.drop("Q2"))
    with pytest.raises(Exception, match='unable to find column "Q2"') as error:
        view.to_polars()
    assert type(error.value).__name__ == "ColumnNotFound"


def test_view_can_be_created_before_loading_data():
    survey = read_triple_s(PATH_SSS)
    view = survey["Q2"]
    with pytest.raises(NoDataError):
        view.to_polars()
    survey.data = read_data(survey.datamap, PATH_ASC)
    assert len(view.to_polars()) == 3
    with pytest.raises(KeyError):
        survey["missing"]


def test_cloned_survey_data_retains_cache():
    original = read_triple_s(PATH_SSS, PATH_ASC)
    data = original.data
    expected = data.to_polars()
    another = Survey(original.datamap, data)
    del original, data
    gc.collect()
    assert_frame_equal(another.data.to_polars(), expected)


@pytest.mark.parametrize("name", ["Q2", "Q3", "Q3.a", "Q1.a", "Q1.b", "WT"])
def test_column_preserves_name_dtype_and_values(survey, name):
    actual = survey[name].to_polars()
    assert isinstance(actual, pl.Series)
    assert_series_equal(actual, survey.data.to_polars().get_column(name))


QUESTION_NAMES = [
    "RESPONDENT_ID", "Q1.a", "Q1.b", "Q2", "Q3", "Q3.a", "Q4", "Q5", "Q6",
    "Q7", "Q8", "WT",
]


@pytest.mark.parametrize("name", QUESTION_NAMES)
def test_question_data_python_values(survey, name):
    question = survey[name]
    expected = survey.data.to_polars()[name].to_list()
    assert question.to_list() == expected
    assert question.data == tuple(expected)
    assert list(question) == expected
    iterator = iter(question)
    assert iter(iterator) is iterator
    assert list(iterator) == expected
    assert next(iterator, "exhausted") == "exhausted"
    assert next(iterator, "exhausted") == "exhausted"
    for index in range(-len(expected), len(expected)):
        assert question[index] == expected[index]


@pytest.mark.parametrize("name", QUESTION_NAMES)
def test_question_data_pandas_export(survey, name):
    result = survey[name].to_pandas()
    assert isinstance(result, pd.Series)
    pd.testing.assert_series_equal(
        result,
        survey.data.to_polars()[name].to_pandas(use_pyarrow_extension_array=True),
    )


@pytest.mark.parametrize("name", ["Q2", "Q3", "Q8"])
def test_question_data_label_exports(survey, name):
    question = survey[name]
    labels = question.response_label
    raw = question.to_list()
    expected = [
        None if value is None else
        [labels[code].value for code in value] if isinstance(value, list) else
        labels[value].value
        for value in raw
    ]
    assert question.to_list(response_label=True) == expected
    assert question.to_polars(response_label=True).to_list() == expected
    pd.testing.assert_series_equal(
        question.to_pandas(response_label=True),
        pl.Series(name, expected).to_pandas(use_pyarrow_extension_array=True),
    )
    survey.response_display = "label"
    assert list(question) == expected
    assert question.data == tuple(expected)
    assert question[0] == expected[0]
    # Question export methods default to raw values, independently of the survey setting.
    assert question.to_list() == raw
    assert question.to_list(response_label=False) == raw


@pytest.mark.parametrize("index", [3, -4, 100, -100])
def test_question_data_index_out_of_bounds(survey, index):
    with pytest.raises(pl.exceptions.OutOfBoundsError):
        survey["Q2"][index]


@pytest.mark.parametrize("index", [None, 1.5, "0", [], slice(None)])
def test_question_data_index_invalid_type(survey, index):
    with pytest.raises(TypeError):
        survey["Q2"][index]


def test_question_data_metadata_shares_datamap(survey):
    question = survey["q2"]
    definition = survey.datamap["Q2"]
    assert question.name == definition.name == "Q2"
    assert question.type == definition.type == "single"
    assert question.label == definition.label
    assert question.range == definition.response.range
    assert question.response_label == definition.response_label
    question.datamap.label = "Changed through the data view"
    assert definition.label.value == question.label.value == "Changed through the data view"
    quantity = survey["WT"]
    survey.datamap["WT"].response.range.end = 15.0
    assert quantity.range.end == 15.0
    assert survey["Q8"].range is None


def test_question_data_python_containers_are_independent(survey):
    question = survey["Q3"]
    expected = question.to_list()
    values = question.to_list()
    values[0].append(999)
    data = question.data
    data[0].append(999)
    labels = question.response_label
    labels[1].value = "Changed copy"
    assert question.to_list() == expected
    assert question.response_label[1].value != "Changed copy"


@pytest.mark.parametrize("method", ["to_list", "to_pandas", "to_polars", "__iter__"])
def test_question_data_exports_require_data(method):
    question = read_triple_s(PATH_SSS)["Q2"]
    with pytest.raises(NoDataError):
        getattr(question, method)()


def test_question_data_direct_construction_is_unsupported():
    with pytest.raises(NotImplementedError):
        QuestionData()
