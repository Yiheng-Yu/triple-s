import polars as pl
import pytest

from triple_s import Survey, SurveyData, read_triple_s


@pytest.fixture
def rows_survey():
    source = read_triple_s("tests/sss-examples/sss-2.0/example1.sss")
    datamap = source.datamap
    for question_name in datamap.record.question_names:
        if question_name not in {"Q2", "Q3"}:
            datamap.record.remove_question(question_name)

    frame = pl.DataFrame({
        "Q2": pl.Series([0, 1, 99, None], dtype=pl.UInt32),
        "Q3": pl.Series([[1, 9], [], [99], None], dtype=pl.List(pl.UInt32)),
    })
    return Survey(datamap, data=SurveyData.from_polars(frame))


def test_row_iteration_label_toggle(rows_survey):
    raw_rows = rows_survey.data.to_polars().to_dicts()
    value_iterator = iter(rows_survey)

    assert rows_survey.response_display == "value"
    assert list(rows_survey) == raw_rows
    assert list(rows_survey.iter_rows()) == raw_rows

    rows_survey.response_display = "label"
    q2_labels = rows_survey["Q2"].response_label
    q3_labels = rows_survey["Q3"].response_label
    expected_label_rows = [
        {
            "Q2": q2_labels[0].value,
            "Q3": [q3_labels[1].value, q3_labels[9].value],
        },
        {"Q2": q2_labels[1].value, "Q3": []},
        {"Q2": "99", "Q3": ["99"]},
        {"Q2": None, "Q3": None},
    ]
    label_iterator = rows_survey.iter_rows()

    assert rows_survey.response_display == "label"
    assert list(rows_survey) == expected_label_rows
    assert [rows_survey[index] for index in range(4)] == expected_label_rows
    assert list(rows_survey.data) == raw_rows

    rows_survey.response_display = "value"

    assert rows_survey.response_display == "value"
    assert list(rows_survey) == raw_rows
    assert list(rows_survey.iter_rows()) == raw_rows
    assert [rows_survey[index] for index in range(4)] == raw_rows
    assert list(value_iterator) == raw_rows
    assert list(label_iterator) == expected_label_rows
