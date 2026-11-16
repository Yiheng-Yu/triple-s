from datetime import date, time

import pandas as pd
import polars as pl
import pytest
from polars.testing import assert_frame_equal

from triple_s import Datamap, NoDataError, Survey, SurveyData, config


@pytest.fixture
def survey():
    datamap = Datamap.from_dict(
        {
            "record": {
                "content": [
                    {
                        "name": name,
                        "question_type": kind,
                        "response": {
                            "values": [
                                {"value": 1, "label": labels[0]},
                                {"value": 2, "label": labels[1]},
                            ]
                        },
                    }
                    for name, kind, labels in [
                        ("choice", "single", ("Yes", "No")),
                        ("group", "single", ("North", "South")),
                        ("multi", "multiple", ("One", "Two")),
                    ]
                ]
            },
        }
    )
    frame = pl.DataFrame(
        {
            "choice": pl.Series([1, 1, 2, 2], dtype=pl.UInt32),
            "group": pl.Series([1, 2, 1, 2], dtype=pl.UInt32),
            "multi": pl.Series([[1, 2], [2], [1], []], dtype=pl.List(pl.UInt32)),
        }
    )
    return Survey(datamap, SurveyData.from_polars(frame))


@pytest.mark.parametrize(
    "row, column, expected",
    [
        (
            "choice",
            "group",
            [
                {"choice": "Base", "Total": None, "North": 2, "South": 2},
                {"choice": "Yes", "Total": 2, "North": 1, "South": 1},
                {"choice": "No", "Total": 2, "North": 1, "South": 1},
            ],
        ),
        (
            "multi",
            "group",
            [
                {"multi": "Base", "Total": None, "North": 2, "South": 2},
                {"multi": "One", "Total": 2, "North": 2, "South": 0},
                {"multi": "Two", "Total": 2, "North": 1, "South": 1},
            ],
        ),
        (
            "choice",
            "multi",
            [
                {"choice": "Base", "Total": None, "One": 2, "Two": 2},
                {"choice": "Yes", "Total": 2, "One": 1, "Two": 2},
                {"choice": "No", "Total": 2, "One": 1, "Two": 0},
            ],
        ),
    ],
)
def test_crosstab_counts(survey, row, column, expected):
    result = survey.crosstab(row, column)
    assert result.columns[0] == row
    assert sorted(result.to_dicts(), key=lambda item: item[row]) == sorted(expected, key=lambda item: item[row])


@pytest.mark.parametrize("format", ["polars", "pandas", config.DataFrameFormat.Polars, config.DataFrameFormat.Pandas])
def test_crosstab_output_format(survey, format):
    result = survey.crosstab("choice", "group", format=format)
    expected = survey.crosstab("choice", "group")
    if format in ("pandas", config.DataFrameFormat.Pandas):
        pd.testing.assert_frame_equal(
            result.sort_values("choice").reset_index(drop=True),
            expected.to_pandas(use_pyarrow_extension_array=True).sort_values("choice").reset_index(drop=True),
        )
    else:
        assert_frame_equal(result.sort("choice"), expected.sort("choice"))


@pytest.mark.parametrize("method", ["crosstab", "response_label_map", "value_counts"])
@pytest.mark.parametrize("format, error", [("invalid", ValueError), (42, TypeError), ([], TypeError)])
def test_analysis_rejects_invalid_format(survey, method, format, error):
    with pytest.raises(error):
        if method == "crosstab":
            survey.crosstab("choice", "group", format=format)
        elif method == "response_label_map":
            survey.response_label_map(format=format)
        else:
            survey["choice"].value_counts(format=format)


@pytest.mark.parametrize("method", ["crosstab", "value_counts"])
def test_analysis_requires_data(survey, method):
    survey.data = SurveyData.empty()
    with pytest.raises(NoDataError):
        if method == "crosstab":
            survey.crosstab("choice", "group")
        else:
            survey["choice"].value_counts()


@pytest.mark.parametrize("row, column", [("missing", "group"), ("choice", "missing")])
def test_crosstab_missing_question(survey, row, column):
    with pytest.raises(Exception, match="missing") as error:
        survey.crosstab(row, column)
    assert type(error.value).__name__ == "ColumnNotFound"


def test_response_label_map_contents(survey):
    result = survey.response_label_map()
    assert result.schema == {
        "question_name": pl.String,
        "response": pl.String,
        "label": pl.String,
    }
    assert set(result.iter_rows()) == {
        ("choice", "1", "Yes"),
        ("choice", "2", "No"),
        ("group", "1", "North"),
        ("group", "2", "South"),
        ("multi", "1", "One"),
        ("multi", "2", "Two"),
    }


def test_response_label_map_without_data(survey):
    expected = survey.response_label_map().sort(["question_name", "response"])
    survey.data = SurveyData.empty()
    assert_frame_equal(survey.response_label_map().sort(["question_name", "response"]), expected)
    empty = Survey(Datamap.from_dict({})).response_label_map()
    assert empty.shape == (0, 3)
    assert empty.columns == ["question_name", "response", "label"]


def test_response_label_map_tracks_label_changes(survey):
    before = survey.response_label_map()
    survey.datamap["choice"].response.get_value(1).label = "Updated"
    after = survey.response_label_map()
    assert after.filter(pl.col("label") == "Updated").to_dicts() == [
        {"question_name": "choice", "response": "1", "label": "Updated"}
    ]
    assert "Updated" not in before["label"].to_list()


@pytest.mark.parametrize("format", ["polars", "pandas", config.DataFrameFormat.Polars, config.DataFrameFormat.Pandas])
def test_response_label_map_format_overrides_default(survey, format):
    survey.dataframe_format = config.DataFrameFormat.Pandas
    assert isinstance(survey.response_label_map(), pd.DataFrame)
    result = survey.response_label_map(format=format)
    expected_type = pd.DataFrame if format in ("pandas", config.DataFrameFormat.Pandas) else pl.DataFrame
    assert isinstance(result, expected_type)
    assert result.shape == (6, 3)
    assert survey.dataframe_format == "pandas"


@pytest.mark.parametrize(
    "question, values, expected",
    [
        (
            "choice",
            [1, 1, 2, None],
            [
                {"choice": None, "label": "*Total*", "count": 3},
                {"choice": 1, "label": "Yes", "count": 2},
                {"choice": 2, "label": "No", "count": 1},
            ],
        ),
        (
            "multi",
            [[1, 2], [2], [], None],
            [
                {"multi": None, "label": "*Total*", "count": 3},
                {"multi": 1, "label": "One", "count": 1},
                {"multi": 2, "label": "Two", "count": 2},
            ],
        ),
    ],
)
def test_value_counts_values_labels_and_nulls(survey, question, values, expected):
    dtype = pl.UInt32 if question == "choice" else pl.List(pl.UInt32)
    survey.data = SurveyData.from_polars(
        pl.DataFrame(
            {
                question: pl.Series(values, dtype=dtype),
            }
        )
    )
    assert survey[question].value_counts().to_dicts() == expected
    survey.response_display = config.ResponseDisplay.Label
    assert survey[question].value_counts().to_dicts() == expected


@pytest.mark.parametrize("format", ["polars", "pandas", config.DataFrameFormat.Polars, config.DataFrameFormat.Pandas])
def test_value_counts_format_and_count_name(survey, format):
    question = survey["choice"]
    survey.dataframe_format = "pandas"
    result = question.value_counts(name="answers", format=format)
    expected = pl.DataFrame(
        {
            "choice": [None, 1, 2],
            "label": ["*Total*", "Yes", "No"],
            "answers": [4, 2, 2],
        }
    )
    if format in ("pandas", config.DataFrameFormat.Pandas):
        assert isinstance(result, pd.DataFrame)
        result = pl.from_pandas(result)
    assert_frame_equal(result, expected, check_dtypes=False)
    assert question.dataframe_format == "pandas"


@pytest.mark.parametrize("kind, first, second", [
    ("date", date(2026, 1, 1), date(2026, 1, 2)),
    ("time", time(12, 30), time(14, 0)),
])
def test_value_counts_temporal_columns(kind, first, second):
    datamap = Datamap.from_dict({"record": {"content": [
        {"name": "Q1", "question_type": kind},
    ]}})
    data = SurveyData.from_polars(pl.DataFrame({"Q1": [first, first, second, None]}))
    result = Survey(datamap, data)["Q1"].value_counts()
    assert result.to_dicts() == [
        {"Q1": None, "label": "*Total*", "count": 3},
        {"Q1": first, "label": None, "count": 2},
        {"Q1": second, "label": None, "count": 1},
    ]
