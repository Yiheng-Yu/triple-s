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
                {"choice": "Null", "choice - Total": 4, "group - North": 2, "group - South": 2},
                {"choice": "Yes", "choice - Total": 2, "group - North": 1, "group - South": 1},
                {"choice": "No", "choice - Total": 2, "group - North": 1, "group - South": 1},
            ],
        ),
        (
            "multi",
            "group",
            [
                {"multi": "Null", "multi - Total": 4, "group - North": 2, "group - South": 2},
                {"multi": "One", "multi - Total": 2, "group - North": 2, "group - South": 0},
                {"multi": "Two", "multi - Total": 2, "group - North": 1, "group - South": 1},
            ],
        ),
        (
            "choice",
            "multi",
            [
                {"choice": "Null", "choice - Total": 4, "multi - One": 2, "multi - Two": 2},
                {"choice": "Yes", "choice - Total": 2, "multi - One": 1, "multi - Two": 2},
                {"choice": "No", "choice - Total": 2, "multi - One": 1, "multi - Two": 0},
            ],
        ),
    ],
)
def test_crosstab_counts(survey, row, column, expected):
    result = survey.crosstab(row, [column])
    assert result.columns[0] == row
    assert result[row][0] == "Null"
    assert sorted(result.to_dicts(), key=lambda item: item[row]) == sorted(expected, key=lambda item: item[row])


@pytest.mark.parametrize("row, labels, north, south", [
    ("choice", ["Yes", "No"], [1, 1], [1, 1]),
    ("multi", ["One", "Two"], [2, 1], [0, 1]),
])
@pytest.mark.parametrize("show_row_total", [False, True])
@pytest.mark.parametrize("show_column_total", [False, True])
def test_crosstab_total_flags(survey, row, labels, north, south, show_row_total, show_column_total):
    expected = pl.DataFrame({
        row: ["Null", *labels],
        f"{row} - Total": [4, 2, 2],
        "group - North": [2, *north],
        "group - South": [2, *south],
    })
    if not show_row_total:
        expected = expected.drop(f"{row} - Total")
    if not show_column_total:
        expected = expected.slice(1)
    result = survey.crosstab(
        row, ["group"], show_row_total=show_row_total, show_column_total=show_column_total,
    )
    assert_frame_equal(result, expected, check_dtypes=False)


@pytest.mark.parametrize("columns", [["group", "multi"], ["multi", "group"]])
def test_crosstab_multiple_columns(survey, columns):
    expected = pl.DataFrame({
        "choice": ["Null", "Yes", "No"],
        "choice - Total": [4, 2, 2],
        "group - North": [2, 1, 1],
        "group - South": [2, 1, 1],
        "multi - One": [2, 1, 1],
        "multi - Two": [2, 2, 0],
    })
    labels = {"group": ["North", "South"], "multi": ["One", "Two"]}
    names = ["choice", "choice - Total"] + [
        f"{column} - {label}" for column in columns for label in labels[column]
    ]
    assert_frame_equal(survey.crosstab("choice", columns), expected.select(names), check_dtypes=False)


def test_crosstab_none_total_flags_use_defaults(survey):
    assert_frame_equal(
        survey.crosstab("choice", ["group"], show_row_total=None, show_column_total=None),
        survey.crosstab("choice", ["group"]),
    )


@pytest.mark.xfail(
    strict=True,
    reason="crosstab currently joins row totals to an empty frame without the row-question column",
)
@pytest.mark.parametrize("kwargs", [{}, {"column": None}, {"column": []}])
def test_crosstab_without_column_questions(survey, kwargs):
    expected = pl.DataFrame({
        "choice": ["Null", "Yes", "No"],
        "choice - Total": [4, 2, 2],
    })
    assert_frame_equal(survey.crosstab("choice", **kwargs), expected, check_dtypes=False)


@pytest.mark.parametrize("column", ["group", 1, [1], [None]])
def test_crosstab_rejects_invalid_columns(survey, column):
    with pytest.raises(TypeError):
        survey.crosstab("choice", column)


@pytest.mark.parametrize("flag", ["show_row_total", "show_column_total"])
@pytest.mark.parametrize("value", ["yes", []])
def test_crosstab_rejects_invalid_total_flags(survey, flag, value):
    with pytest.raises(TypeError):
        survey.crosstab("choice", ["group"], **{flag: value})


@pytest.mark.parametrize("format, expected_type", [("polars", pl.DataFrame), ("pandas", pd.DataFrame)])
def test_crosstab_uses_survey_default_format(survey, format, expected_type):
    survey.dataframe_format = format
    assert isinstance(survey.crosstab("choice", ["group"]), expected_type)


@pytest.mark.parametrize("format", ["polars", "pandas", config.DataFrameFormat.Polars, config.DataFrameFormat.Pandas])
def test_crosstab_output_format(survey, format):
    survey.dataframe_format = "pandas" if format in ("polars", config.DataFrameFormat.Polars) else "polars"
    result = survey.crosstab("choice", ["group"], format=format)
    expected = survey.crosstab("choice", ["group"], format="polars")
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
            survey.crosstab("choice", ["group"], format=format)
        elif method == "response_label_map":
            survey.response_label_map(format=format)
        else:
            survey["choice"].value_counts(output=format)


@pytest.mark.parametrize("method", ["crosstab", "value_counts"])
def test_analysis_requires_data(survey, method):
    survey.data = SurveyData.empty()
    with pytest.raises(NoDataError):
        if method == "crosstab":
            survey.crosstab("choice", ["group"])
        else:
            survey["choice"].value_counts()


@pytest.mark.parametrize("row, column", [("missing", "group"), ("choice", "missing")])
def test_crosstab_missing_question(survey, row, column):
    with pytest.raises(Exception, match="missing") as error:
        survey.crosstab(row, [column])
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
                {"label": "Total", "total": 3},
                {"label": "Yes", "total": 2},
                {"label": "No", "total": 1},
            ],
        ),
        (
            "multi",
            [[1, 2], [2], [], None],
            [
                {"label": "Total", "total": 3},
                {"label": "One", "total": 1},
                {"label": "Two", "total": 2},
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


@pytest.mark.parametrize("question, values, dtype, base", [
    ("choice", [], pl.UInt32, 0),
    ("choice", [None, None], pl.UInt32, 0),
    ("multi", [], pl.List(pl.UInt32), 0),
    ("multi", [None, None], pl.List(pl.UInt32), 0),
    ("multi", [[], [], None], pl.List(pl.UInt32), 2),
])
@pytest.mark.parametrize("include_code_value", [False, True])
def test_value_counts_without_selected_answers(survey, question, values, dtype, base, include_code_value):
    survey.data = SurveyData.from_polars(
        pl.DataFrame({question: pl.Series(values, dtype=dtype)})
    )
    expected = {"label": "Total", "total": base}
    if include_code_value:
        expected["code"] = None
    assert survey[question].value_counts(include_code_value=include_code_value).to_dicts() == [expected]


@pytest.mark.parametrize("output", ["polars", "pandas", config.DataFrameFormat.Polars, config.DataFrameFormat.Pandas])
def test_value_counts_output_and_column_names(survey, output):
    survey.dataframe_format = "polars" if output in ("pandas", config.DataFrameFormat.Pandas) else "pandas"
    question = survey["choice"]
    result = question.value_counts(
        include_question_name=True, include_code_value=True, colname_stats="answers",
        colname_label="answer", colname_code="value", output=output,
    )
    expected = pl.DataFrame(
        {
            "question": ["choice", None, None],
            "value": [None, 1, 2],
            "answer": ["Total", "Yes", "No"],
            "answers": [4, 2, 2],
        }
    )
    if output in ("pandas", config.DataFrameFormat.Pandas):
        assert isinstance(result, pd.DataFrame)
        result = pl.from_pandas(result)
    assert_frame_equal(result, expected, check_dtypes=False)
    assert question.dataframe_format == survey.dataframe_format


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
        {"label": "Total", "total": 3},
        {"label": first.isoformat(), "total": 2},
        {"label": second.isoformat(), "total": 1},
    ]


@pytest.mark.parametrize("question, labels", [("choice", ["Yes", "No"]), ("multi", ["One", "Two"])])
@pytest.mark.parametrize("include_question_name", [False, True])
@pytest.mark.parametrize("include_code_value", [False, True])
def test_value_counts_optional_columns(survey, question, labels, include_question_name, include_code_value):
    columns = {}
    if include_question_name:
        columns["question"] = pl.Series([question, None, None])
    if include_code_value:
        columns["code"] = pl.Series([None, 1, 2], dtype=pl.UInt32)
    columns["label"] = pl.Series(["Total", *labels])
    columns["total"] = pl.Series([4, 2, 2], dtype=pl.UInt32)
    result = survey[question].value_counts(include_question_name=include_question_name, include_code_value=include_code_value)
    assert_frame_equal(result, pl.DataFrame(columns))


@pytest.mark.parametrize("output, expected_type", [("polars", pl.DataFrame), ("pandas", pd.DataFrame)])
def test_value_counts_uses_survey_default_output(survey, output, expected_type):
    survey.dataframe_format = output
    question = survey["choice"]
    assert isinstance(question.value_counts(), expected_type)
    assert isinstance(question.value_counts(output=None), expected_type)


@pytest.mark.parametrize("option", ["include_question_name", "include_code_value", "colname_stats", "colname_label", "colname_code"])
def test_value_counts_none_options_use_defaults(survey, option):
    question = survey["choice"]
    kwargs = {"include_question_name": True, "include_code_value": True} if option.startswith("colname_") else {}
    assert_frame_equal(
        question.value_counts(**{**kwargs, option: None}),
        question.value_counts(**kwargs),
    )


@pytest.mark.parametrize("option", ["include_question_name", "include_code_value"])
@pytest.mark.parametrize("value", ["yes", []])
def test_value_counts_rejects_invalid_flags(survey, option, value):
    with pytest.raises(TypeError):
        survey["choice"].value_counts(**{option: value})


@pytest.mark.parametrize("option", ["colname_stats", "colname_label", "colname_code"])
@pytest.mark.parametrize("value", [42, []])
def test_value_counts_rejects_invalid_column_names(survey, option, value):
    with pytest.raises(TypeError):
        survey["choice"].value_counts(**{option: value})


@pytest.mark.parametrize("option", ["name", "format"])
def test_value_counts_rejects_old_keywords(survey, option):
    with pytest.raises(TypeError, match="unexpected keyword argument"):
        survey["choice"].value_counts(**{option: "answers"})


def test_value_counts_unlabelled_codes(survey):
    survey.data = SurveyData.from_polars(pl.DataFrame({
        "choice": pl.Series([3, 1, 3, None], dtype=pl.UInt32),
    }))
    assert survey["choice"].value_counts(include_code_value=True).to_dicts() == [
        {"code": None, "label": "Total", "total": 3},
        {"code": 1, "label": "Yes", "total": 1},
        {"code": 3, "label": "3", "total": 2},
    ]
