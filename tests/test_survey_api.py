from triple_s import Survey, read_triple_s
import pytest

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC)


def test_read_file_success(survey):
    assert survey.has_data()


def test_survey_field_mutation(survey):
    datamap = survey.datamap
    datamap.title = "haha"
    assert survey.title == "haha"

    survey.title = "yes"
    assert datamap.title == "yes"
