import pandas as pd
import polars as pl
import pytest
from polars.testing import assert_frame_equal

from triple_s import NoDataError, config, read_triple_s


@pytest.fixture
def survey():
    return read_triple_s(
        "tests/sss-examples/sss-2.0/example1.sss",
        "tests/sss-examples/sss-2.0/example1.asc",
    )


def test_survey_name_setter_shares_metadata(survey):
    for name in ("Updated survey", None):
        survey.name = name
        assert survey.name == survey.info.name == survey.datamap.info.name == name


@pytest.mark.parametrize("index", [-1, -2, -3])
def test_survey_negative_row_index(survey, index):
    assert survey[index] == survey.data.to_polars().row(index, named=True)


@pytest.mark.parametrize("index", [3, -4, 100, -100])
def test_survey_row_index_out_of_bounds(survey, index):
    with pytest.raises(IndexError):
        survey[index]


@pytest.mark.parametrize("index", [None, 1.5, [], {}, slice(None)])
def test_survey_index_rejects_invalid_types(survey, index):
    with pytest.raises(TypeError):
        survey[index]


@pytest.mark.parametrize("field, value, expected", [
    ("dataframe_format", config.DataFrameFormat.Pandas, "pandas"),
    ("dataframe_format", config.DataFrameFormat.Polars, "polars"),
    ("response_display", config.ResponseDisplay.Label, "label"),
    ("response_display", config.ResponseDisplay.Value, "value"),
])
def test_survey_settings_accept_enum_members(survey, field, value, expected):
    original = getattr(survey, field)
    view = survey["Q2"]
    iterated_view = next(survey.iter_questions())
    setattr(survey, field, value)
    assert getattr(survey, field) == expected
    assert getattr(view, field) == getattr(iterated_view, field) == original
    assert getattr(survey["Q2"], field) == expected
    assert getattr(next(survey.iter_questions()), field) == expected


@pytest.mark.parametrize("field", ["dataframe_format", "response_display"])
@pytest.mark.parametrize("value, error", [
    (None, TypeError), (3.14, TypeError), ([], TypeError),
    (lambda value: value, TypeError), ("", ValueError), ("invalid", ValueError),
])
def test_survey_settings_reject_each_invalid_value(survey, field, value, error):
    before = getattr(survey, field)
    with pytest.raises(error):
        setattr(survey, field, value)
    assert getattr(survey, field) == before


@pytest.mark.parametrize("method", ["to_polars", "to_pandas"])
@pytest.mark.parametrize("display", ["value", "label"])
def test_survey_exports_respect_display_and_explicit_override(survey, method, display):
    raw = survey.data.to_polars()
    labelled_columns = []
    for definition in survey.datamap:
        if definition.type not in ("single", "multiple"):
            continue
        labels = definition.response_label
        values = [
            None if value is None else
            [labels[code].value for code in value] if isinstance(value, list) else
            labels[value].value
            for value in raw[definition.name].to_list()
        ]
        labelled_columns.append(pl.Series(definition.name, values))
    labelled = raw.with_columns(labelled_columns)
    survey.response_display = display
    for response_label in (None, False, True):
        kwargs = {} if response_label is None else {"response_label": response_label}
        result = getattr(survey, method)(**kwargs)
        show_labels = display == "label" if response_label is None else response_label
        expected = labelled if show_labels else raw
        if show_labels:
            # Check categorical labels; other columns follow the current export coercions.
            names = [column.name for column in labelled_columns]
            expected = expected.select(names)
            result = result[names] if method == "to_pandas" else result.select(names)
        if method == "to_pandas":
            pd.testing.assert_frame_equal(
                result, expected.to_pandas(use_pyarrow_extension_array=True)
            )
        else:
            assert_frame_equal(result, expected)
        assert survey.response_display == display


@pytest.mark.parametrize("method", ["to_polars", "to_pandas", "iter_rows", "__iter__"])
def test_survey_operations_require_loaded_data(method):
    survey = read_triple_s("tests/sss-examples/sss-2.0/example1.sss")
    with pytest.raises(NoDataError):
        getattr(survey, method)()
