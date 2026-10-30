import pytest
from triple_s import read_triple_s

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC)


def test_read_file_success(survey):
    assert survey.has_data()


def test_survey_title_mutation(survey):
    datamap = survey.datamap
    datamap.title = "haha"
    assert survey.title == "haha"

    survey.title = "yes"
    assert datamap.title == "yes"


@pytest.mark.parametrize(
    "attribute",
    [
        pytest.param("title", id="title"),
        pytest.param("name", id="name"),
        pytest.param("user", id="user"),
        pytest.param("origin", id="origin"),
        pytest.param("user", id="user"),
        pytest.param("date", id="date"),
        pytest.param("time", id="time"),
        pytest.param("style", id="style"),
        pytest.param("standard", id="standard"),
    ],
)
def test_survey_info_setter(survey, attribute):
    info = survey.datamap.info

    # basic mutation check
    setattr(survey.datamap.info, attribute, "test 1")
    assert getattr(survey.datamap.info, attribute) == "test 1"


@pytest.mark.parametrize(
    "attribute",
    [
        pytest.param("title", id="title"),
        pytest.param("name", id="name"),
        pytest.param("user", id="user"),
        pytest.param("origin", id="origin"),
        pytest.param("user", id="user"),
        pytest.param("date", id="date"),
        pytest.param("time", id="time"),
        pytest.param("style", id="style"),
        pytest.param("standard", id="standard"),
    ],
)
def test_survey_info_mutation_sync(survey, attribute):
    info = survey.datamap.info
    datamap = survey.datamap

    # mutate info -> check if survey.datamap updates
    setattr(info, attribute, "test")
    assert getattr(survey.datamap.info, attribute) == "test"
    assert getattr(datamap.info, attribute) == "test"

    setattr(info, attribute, None)
    assert getattr(survey.datamap.info, attribute) is None
    assert getattr(datamap.info, attribute) is None

    # mutate survey.datamap -> check if `info` updates
    setattr(survey.datamap.info, attribute, "value 2")
    assert getattr(info, attribute) == "value 2"
    assert getattr(datamap.info, attribute) == "value 2"

    setattr(survey.datamap.info, attribute, None)
    assert getattr(info, attribute) is None
    assert getattr(datamap.info, attribute) is None

    # mutate datamap -> check if `info` and `survey.datamap.info` updates
    setattr(datamap.info, attribute, "value 3")
    assert getattr(info, attribute) == "value 3"
    assert getattr(survey.datamap.info, attribute) == "value 3"
    setattr(datamap.info, attribute, None)
    assert getattr(info, attribute) is None
    assert getattr(survey.datamap.info, attribute) is None
