import gc

import polars as pl
import pytest

from triple_s import Datamap, Location, Question, Survey, SurveyData, read_triple_s


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


@pytest.mark.parametrize("count", [0, 1, 255, 256, 257, 512, 513])
def test_row_iteration_across_buffers(rows_survey, count):
    base = rows_survey.data.to_polars()
    frame = pl.concat([base] * max(1, (count + 3) // 4), rechunk=False).head(count)
    rows_survey.data = SurveyData.from_polars(frame)
    expected = frame.to_dicts()
    for iterator in [iter(rows_survey), iter(rows_survey.data)]:
        assert list(iterator) == expected
        assert next(iterator, "exhausted") == "exhausted"
        assert next(iterator, "exhausted") == "exhausted"


def test_new_iterators_observe_data_and_label_changes(rows_survey):
    frame = pl.DataFrame({
        "Q2": pl.Series([0] * 600, dtype=pl.UInt32),
        "Q3": pl.Series([[1]] * 600, dtype=pl.List(pl.UInt32)),
    })
    rows_survey.data = SurveyData.from_polars(frame)
    raw = frame.to_dicts()
    values = iter(rows_survey)
    data_values = iter(rows_survey.data)
    rows_survey.response_display = "label"
    labels = iter(rows_survey)
    old_q2_label = rows_survey["Q2"].response_label[0].value
    old_q3_label = rows_survey["Q3"].response_label[1].value
    old_label_row = {"Q2": old_q2_label, "Q3": [old_q3_label]}
    assert next(values) == raw[0]
    assert next(data_values) == raw[0]
    assert next(labels) == old_label_row

    rows_survey.datamap.record["Q2"].response.get_value(0).label = "Updated label"
    assert next(iter(rows_survey))["Q2"] == "Updated label"
    rows_survey.response_display = "value"
    replacement = frame.head(2).with_columns(pl.lit(1, dtype=pl.UInt32).alias("Q2"))
    rows_survey.data = SurveyData.from_polars(replacement)
    assert list(rows_survey) == replacement.to_dicts()
    assert list(rows_survey.data) == replacement.to_dicts()
    assert list(values) == raw[1:]
    assert list(data_values) == raw[1:]
    assert list(labels) == [old_label_row] * 599


def test_row_iterators_own_their_source_and_return_independent_values():
    survey = read_triple_s(
        "tests/sss-examples/sss-2.0/example1.sss",
        "tests/sss-examples/sss-2.0/example1.asc",
    )
    expected = survey.data.to_polars().to_dicts()
    values = iter(survey)
    data_values = iter(survey.data)
    del survey
    gc.collect()
    assert list(values) == expected
    assert list(data_values) == expected


def test_mutating_a_yielded_row_does_not_change_other_iterators(rows_survey):
    values = iter(rows_survey)
    row = next(values)
    row["Q2"] = 42
    row["Q3"].append(42)
    assert next(iter(rows_survey)) == {"Q2": 0, "Q3": [1, 9]}
    assert next(values) == {"Q2": 1, "Q3": []}


def test_wide_row_buffers_preserve_column_order_and_values():
    names = [f"Q{i}" for i in range(3000)]
    datamap = Datamap.from_dict({
        "record": {"content": [
            Question(name, "quantity", location=Location(i + 1))
            for i, name in enumerate(names)
        ]}
    })
    values = pl.Series("Q0", range(50), dtype=pl.Float64)
    frame = pl.DataFrame([values.rename(name) for name in names])
    survey = Survey(datamap, data=SurveyData.from_polars(frame))
    for iterator in [iter(survey), iter(survey.data)]:
        count = 0
        for index, row in enumerate(iterator):
            assert len(row) == 3000
            assert row["Q0"] == row["Q2999"] == float(index)
            if index in (0, 49):
                assert list(row) == names
            count += 1
        assert count == 50
