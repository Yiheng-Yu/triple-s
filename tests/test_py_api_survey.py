import polars
import pandas

import pytest
from triple_s import read_data, NoDataError, read_triple_s

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/stackoverflow_2026.sss"
PATH_CSV = "tests/sss-examples/sss-2.0/stackoverflow_2026.csv"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_CSV)


def test_read_only_datamap():
    survey = read_triple_s(PATH_SSS_EXAMPLE_1)
    assert survey.has_data() == False
    assert survey.num_response == 0


def test_read_file_success(survey):
    assert survey.has_data()

    print()
    print("Survey datamap repr:")
    print(survey.datamap)

    print()
    print("Survey data repr:")
    print(survey.data)

    print()
    print("Survet data:")
    print(survey["CodeMain"])


@pytest.mark.parametrize("index", [0, 1, 2])
def test_indexing_individual_responses(survey, index):
    response = survey[index]
    assert response == survey.data.to_polars().row(index, named=True)


# def test_survey_iter(survey):
#     df = survey.data.to_polars()
#     for index, entry in enumerate(survey.iter_rows()):
#         assert entry == df.row(index, named=True)


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

    data = read_data(survey.datamap, PATH_CSV)
    assert data.has_data()
    assert not survey.has_data()


def test_value_counts(survey):
    for question in survey.iter_questions():
        summary = question.value_counts(name="what")
        assert summary.columns[-1] == "what"

def test_set_dataframe_format_config_success(survey):
    assert survey.dataframe_format == 'polars'   # the default
    
    for question in survey.iter_questions():
        assert question.dataframe_format == 'polars'
        assert type(question.value_counts()) == polars.DataFrame
    
    survey.dataframe_format = 'pandas'
    assert survey.dataframe_format == 'pandas'
    for question in survey.iter_questions():
        assert question.dataframe_format == 'pandas'
        assert type(question.value_counts()) == pandas.DataFrame


def test_set_dataframe_format_config_fail(survey):
    with pytest.raises(TypeError):
        survey.dataframe_format = None
        survey.dataframe_format = 3.14
        survey.dataframe_format = []
        survey.dataframe_format = lambda s: s
    
    with pytest.raises(ValueError):
        survey.dataframe_format = ""
        survey.dataframe_format = "pppppppolars"


def test_set_flag_toggle_response_display_success(survey):
    assert survey.response_display == "value"
    
    for question in survey.iter_questions():
        assert question.response_display == 'value'
        
    survey.response_display = "label"
    assert survey.response_display == "label"
    for question in survey.iter_questions():
        assert question.response_display == 'label'
        

def test_set_flag_toggle_response_display_fail(survey):
    with pytest.raises(TypeError):
        survey.response_display = None
        survey.response_display = []
        survey.response_display = lambda s: s
    
    with pytest.raises(ValueError):
        survey.response_display = ""
        survey.response_display = "pppppppolars"


def test_print_one_response_as_label(survey):
    survey.response_display = "label"
    assert survey.response_display == "label"
    
    for response in survey:
        assert isinstance(response, dict)
        print(response)
        break
        