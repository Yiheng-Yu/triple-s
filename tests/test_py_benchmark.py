import pytest
from triple_s import read_triple_s

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/stackoverflow_2026.sss"
PATH_CSV = "tests/sss-examples/sss-2.0/stackoverflow_2026.csv"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_CSV)


def read_stack_overflow_survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_CSV)


def dump_to_polars(survey):
    return survey.data.to_polars()


def iter_rows_with_value(survey):
    survey.response_display = "value"
    for row in survey.iter_rows():
        _ = row


def iter_rows_with_label(survey):
    survey.response_display = "label"

    for row in survey.iter_rows():
        _ = row


def value_counts(survey):
    for question in survey.iter_questions():
        _ = question.value_counts()

# =========== tests ===========


def test_read_triple_s(benchmark):
    _ = benchmark(read_stack_overflow_survey)


def test_dump_to_polars(benchmark, survey):
    _ = benchmark(dump_to_polars, survey)


def test_iter_rows_with_value(benchmark, survey):
    _ = benchmark(iter_rows_with_value, survey)


def test_iter_rows_with_labbel(benchmark, survey):
    _ = benchmark(iter_rows_with_label, survey)


def test_value_counts(benchmark, survey):
    _ = benchmark(value_counts, survey)
