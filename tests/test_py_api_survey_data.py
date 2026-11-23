from datetime import date, time

import pandas as pd
import polars as pl
import pytest
from polars.testing import assert_frame_equal

from triple_s import NoDataError, SurveyData


@pytest.fixture
def frame():
    return pl.DataFrame({
        "number": [1.0, None, 3.0],
        "text": ["hello", "世界", None],
        "choices": [[1, 2], [], None],
        "logical": [True, False, None],
        "date": [date(2026, 1, 1), None, date(2026, 1, 3)],
        "time": [time(12, 30), None, time(14, 0)],
    })


def test_survey_data_empty_metadata():
    data = SurveyData.empty()
    assert not data.has_data()
    assert data.num_response == 0
    assert data.question_names == []


@pytest.mark.parametrize("method", ["to_polars", "to_pandas", "describe", "__iter__"])
def test_survey_data_empty_operations_require_data(method):
    with pytest.raises(NoDataError, match="Data not loaded"):
        getattr(SurveyData.empty(), method)()


@pytest.mark.parametrize("rows", [0, 3])
def test_survey_data_from_polars_roundtrip(frame, rows):
    frame = frame.head(rows)
    data = SurveyData.from_polars(frame)
    assert data.has_data()
    assert data.num_response == rows
    assert data.question_names == frame.columns
    assert_frame_equal(data.to_polars(), frame)
    pd.testing.assert_frame_equal(
        data.to_pandas(), frame.to_pandas(use_pyarrow_extension_array=True)
    )
    assert list(data) == frame.to_dicts()


def test_survey_data_describe(frame):
    result = SurveyData.from_polars(frame).describe()
    assert isinstance(result, pl.DataFrame)
    assert_frame_equal(result, frame.describe())
    stats = dict(result.select("statistic", "number").iter_rows())
    assert stats["count"] == 2
    assert stats["null_count"] == 1
    assert stats["mean"] == 2
    assert stats["min"] == 1
    assert stats["max"] == 3


def test_survey_data_exports_are_independent(frame):
    data = SurveyData.from_polars(frame)
    exported = data.to_polars()
    exported.replace_column(0, pl.Series("number", [99.0] * 3))
    frame.replace_column(0, pl.Series("number", [-1.0] * 3))
    assert data.to_polars()["number"].to_list() == [1.0, None, 3.0]
    assert data.question_names[0] == "number"


@pytest.mark.parametrize("value", [None, [], {"number": [1]}, pl.Series([1])])
def test_survey_data_from_polars_rejects_other_types(value):
    with pytest.raises(AttributeError, match="get_columns"):
        SurveyData.from_polars(value)


def test_survey_data_from_pandas():
    df = pd.DataFrame(
        {
            "num_legs": [2, 4, 8, 0],
            "num_wings": [2, 0, 0, 0],
            "num_specimen_seen": [10, 2, 1, 8],
        },
        index=["falcon", "dog", "spider", "fish"],
    )
    df.index.name = "animal"
    data = SurveyData.from_pandas(df, include_index=True)
    assert (data['animal'].to_pandas() == df.index).all()


def test_survey_data_indexing():
    df = pd.DataFrame(
        {
            "num_legs": [2, 4, 8, 0],
            "num_wings": [2, 0, 0, 0],
            "num_specimen_seen": [10, 2, 1, 8],
        },
        index=["falcon", "dog", "spider", "fish"],
    )
    df.index.name = "animal"
    data = SurveyData.from_pandas(df, include_index=True)
    
    assert (data['animal'].to_pandas() == df.index).all()
    for (index, row) in df.reset_index().iterrows():
        row = row.to_dict()
        assert data[index] == row