import gc

import polars as pl
import pytest
from polars.testing import assert_frame_equal, assert_series_equal
from triple_s import NoDataError, QuestionData, Survey, SurveyData, read_triple_s

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
    survey.data = survey.datamap.load_data(PATH_ASC)
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
