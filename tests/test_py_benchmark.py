import random

import pytest
from triple_s import read_triple_s

PATH_SSS_EXAMPLE_STACK_OVERFLOW = "tests/sss-examples/sss-2.0/stackoverflow_2026.sss"
PATH_CSV_EXAMPLE_STACK_OVERFLOW = "tests/sss-examples/sss-2.0/stackoverflow_2026.csv"

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.asc"


REPEAT = 100
MAX_ROW_ITER = 250


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_STACK_OVERFLOW, PATH_CSV_EXAMPLE_STACK_OVERFLOW)


# =============================================================================
# Read data
# =============================================================================
def read_stack_overflow_survey():
    return read_triple_s(PATH_SSS_EXAMPLE_STACK_OVERFLOW, PATH_CSV_EXAMPLE_STACK_OVERFLOW)


def test_read_triple_s_from_csv(benchmark):
    benchmark.pedantic(
        read_stack_overflow_survey,
        iterations=3,
    )


def read_sss_example_asc():
    _ = read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC_EXAMPLE_1)


def test_read_sss_example_asc(benchmark):
    benchmark.pedantic(read_sss_example_asc, iterations=100, rounds=50)


# =============================================================================
# Export to polars/ pandas
# =============================================================================


def dump_to_polars(survey):
    return survey.data.to_polars()


def test_dump_to_polars_response_value(benchmark, survey):
    survey.response_display = "value"
    _ = benchmark(dump_to_polars, survey)


def test_dump_to_polars_response_label(benchmark, survey):
    survey.response_display = "label"
    _ = benchmark(dump_to_polars, survey)


# =============================================================================
# Iter rows
# =============================================================================


def iter_rows_with_value(survey):
    survey.response_display = "value"
    for index, row in enumerate(survey.iter_rows()):
        if index <= MAX_ROW_ITER:
            _ = row
        else:
            return


def test_iter_rows_with_value(benchmark, survey):
    _ = benchmark(iter_rows_with_value, survey)


def iter_rows_with_label(survey):
    survey.response_display = "label"
    for index, row in enumerate(survey.iter_rows()):
        if index <= MAX_ROW_ITER:
            _ = row
        else:
            return


def test_iter_rows_with_labbel(benchmark, survey):
    _ = benchmark(iter_rows_with_label, survey)


# =============================================================================
# Get rows by index
# =============================================================================


def get_row_from_index(survey, index_range):
    index = random.randint(0, index_range)
    _ = survey[index]


def test_get_row_with_value(benchmark, survey):
    survey.response_display = "value"
    index_range = survey.num_response - 1
    benchmark.pedantic(
        get_row_from_index,
        args=(survey, index_range),
        rounds=REPEAT,
        iterations=10,
    )


def test_get_row_with_label(benchmark, survey):
    survey.response_display = "label"
    index_range = survey.num_response - 1
    benchmark.pedantic(
        get_row_from_index,
        args=(survey, index_range),
        rounds=REPEAT,
        iterations=10,
    )


# =============================================================================
# Iter by column
# =============================================================================


def iter_survey_data(survey):
    for row in survey.data:
        _ = row


def test_iter_survey_data_instance(benchmark, survey):
    _ = benchmark(iter_survey_data, survey)


def iter_questions(survey):
    for question in survey.iter_questions():
        _ = question


def test_iter_questions(benchmark, survey):
    _ = benchmark(iter_questions, survey)


# =============================================================================
# Running analysis
# =============================================================================
def value_counts(survey):
    for question in survey.iter_questions():
        _ = question.value_counts()


def test_value_counts(benchmark, survey):
    _ = benchmark(value_counts, survey)
