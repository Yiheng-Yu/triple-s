import polars as pl
import pytest

from triple_s import Survey, SurveyData, read_triple_s


@pytest.fixture
def rows_survey():
    source = read_triple_s("tests/sss-examples/sss-2.0/example1.sss")
    frame = pl.DataFrame({
        "Q2": pl.Series([0, 1, 99, None], dtype=pl.UInt32),
        "Q3": pl.Series([[1, 9], [], [99], None], dtype=pl.List(pl.UInt32)),
        "extra": ["a", "b", None, "d"],
    })
    return Survey(source.datamap, data=SurveyData.from_polars(frame))


def test_row_iteration_label_toggle(rows_survey):
    survey = rows_survey
    raw = survey.data.to_polars().to_dicts()
    values = iter(survey)
    survey.response_display = "label"
    labels = survey.iter_rows()
    expected = [dict(row) for row in raw]
    expected[0]["Q2"] = "No, this is the first visit"
    # Use the public response mapping for the fixture's exact label text.
    expected[1]["Q2"] = survey["Q2"].response_label[1].value
    mapping = survey["Q3"].response_label
    expected[0]["Q3"] = [mapping[value].value for value in [1, 9]]
    assert list(survey) == expected
    assert [survey[i] for i in range(4)] == expected
    assert survey[-1] == expected[-1]
    assert list(survey.data) == raw
    survey.response_display = "value"
    assert list(labels) == expected
    assert list(values) == raw
    assert list(survey.iter_rows()) == raw
    assert [survey[i] for i in range(4)] == raw
    assert list(labels) == []
    assert list(values) == []
