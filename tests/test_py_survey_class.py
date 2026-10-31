from re import L
import pprint

import pytest
from triple_s import NoDataError, read_triple_s

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC)


def test_read_only_datamap():
    survey = read_triple_s(PATH_SSS_EXAMPLE_1)
    assert survey.has_data() == False
    assert survey.num_response == 0


def test_read_file_success(survey):
    assert survey.has_data()
    assert survey.num_response == 3
    assert survey.name == "SP5201-1"
    assert survey.title == "Historic House Exit Survey<br/>First Wave"

    print()
    print("Survey datamap repr:")
    print(survey.datamap)

    print()
    print("Survey data repr:")
    print(survey.data)
    
    print()
    print("Survey iter:")
    print(survey.__iter__())


@pytest.mark.parametrize("index", [0, 1, 2])
def test_indexing_individual_responses(survey, index):
    response = survey[index]
    assert response == survey.data.to_polars().row(index, named=True)


def test_survey_iter(survey):
    df = survey.data.to_polars()
    for index, entry in enumerate(survey):
        assert entry == df.row(index, named=True)


def test_survey_title_mutation(survey):
    datamap = survey.datamap
    datamap.title = "haha"
    assert survey.title == "haha"
    assert survey.info.title == "haha"

    survey.title = "yes"
    assert datamap.title == "yes"
    assert survey.info.title == "yes"


@pytest.mark.parametrize(
    "attribute",
    [
        pytest.param("title", id="title"),
        pytest.param("name", id="name"),
        pytest.param("user", id="user"),
        pytest.param("origin", id="origin"),
        pytest.param("date", id="date"),
        pytest.param("time", id="time"),
        pytest.param("style", id="style"),
        pytest.param("standard", id="standard"),
    ],
)
def test_survey_info_setter(survey, attribute):
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
        pytest.param("date", id="date"),
        pytest.param("time", id="time"),
        pytest.param("style", id="style"),
        pytest.param("standard", id="standard"),
    ],
)
def test_survey_info_mutation_sync(survey, attribute):
    info_direct = survey.datamap.info
    datamap = survey.datamap

    # mutate info -> check if survey.datamap updates
    setattr(info_direct, attribute, "test")
    assert getattr(survey.datamap.info, attribute) == "test"
    assert getattr(survey.info, attribute) == "test"
    assert getattr(datamap.info, attribute) == "test"

    setattr(info_direct, attribute, None)
    assert getattr(survey.datamap.info, attribute) is None
    assert getattr(survey.info, attribute) is None
    assert getattr(datamap.info, attribute) is None

    # mutate survey.datamap -> check if `info` updates
    setattr(survey.datamap.info, attribute, "value 2")
    assert getattr(info_direct, attribute) == "value 2"
    assert getattr(datamap.info, attribute) == "value 2"

    setattr(survey.datamap.info, attribute, None)
    assert getattr(info_direct, attribute) is None
    assert getattr(datamap.info, attribute) is None

    # mutate datamap -> check if `info` and `survey.datamap.info` updates
    setattr(datamap.info, attribute, "value 3")
    assert getattr(info_direct, attribute) == "value 3"
    assert getattr(survey.datamap.info, attribute) == "value 3"
    setattr(datamap.info, attribute, None)
    assert getattr(info_direct, attribute) is None
    assert getattr(survey.datamap.info, attribute) is None

    # mutate datamap -> check if `info` and `survey.datamap.info` updates
    setattr(survey.info, attribute, "value 3")
    assert getattr(info_direct, attribute) == "value 3"
    assert getattr(survey.datamap.info, attribute) == "value 3"
    setattr(survey.info, attribute, None)
    assert getattr(info_direct, attribute) is None
    assert getattr(survey.datamap.info, attribute) is None


def test_survey_without_data():
    survey = read_triple_s(PATH_SSS_EXAMPLE_1)
    assert not survey.has_data()
    assert not survey.data.has_data()
    with pytest.raises(NoDataError):
        survey.data.to_polars()

    data = survey.datamap.load_data(PATH_ASC)
    assert data.has_data()
    assert not survey.has_data()
