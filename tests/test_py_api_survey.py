
import pandas
import polars as pl
import pytest
from triple_s import Code, Datamap, NoDataError, Question, Survey, SurveyData, read_data, read_triple_s

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_DATA = "tests/sss-examples/sss-2.0/example1.asc"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_DATA)


@pytest.mark.parametrize("index", [0, 1, 2])
def test_indexing_individual_responses(survey, index):
    response = survey[index]
    assert response == survey.data.to_polars().row(index, named=True)


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
    assert survey.num_response == 0
    assert not survey.data.has_data()
    with pytest.raises(NoDataError):
        survey.data.to_polars()

    data = read_data(survey.datamap, PATH_DATA)
    assert data.has_data()
    assert not survey.has_data()


def test_set_dataframe_format_config_success(survey):
    assert survey.dataframe_format == 'polars'   # the default
    
    for question in survey.iter_questions():
        assert question.dataframe_format == 'polars'
        assert type(question.value_counts()) == pl.DataFrame
    
    survey.dataframe_format = 'pandas'
    assert survey.dataframe_format == 'pandas'
    for question in survey.iter_questions():
        assert question.dataframe_format == 'pandas'
        assert type(question.value_counts()) == pandas.DataFrame


def test_set_flag_toggle_response_display_success(survey):
    assert survey.response_display == "value"
    
    for question in survey.iter_questions():
        assert question.response_display == 'value'
        
    survey.response_display = "label"
    assert survey.response_display == "label"
    for question in survey.iter_questions():
        assert question.response_display == 'label'

def test_survey_crosstab_input_types(survey):
    name = "Q2"
    column = "Q3"
    _ = survey.crosstab(name, column)
    _ = survey.crosstab(name, [column])


@pytest.mark.parametrize(
    "column, ignore",
    [
        pytest.param(None, ["Q3"], id="(col: None | ignore: list)"),
        pytest.param("Q2", ["Q3"], id="(col: str | ignore: list)"),
        pytest.param(["Q2"], ["Q3"], id="(col: list | ignore: list)"),
        pytest.param(["Q2"], None, id="(col: list | ignore: None)"),
        pytest.param(["Q2"], "Q3", id="(col: list | ignore: str)"),
        
        pytest.param("Q2", "Q3", id="(col: str | ignore: str)"),
        pytest.param(["Q2", "Q3"], "Q3", id="(col: list[2] | ignore: str)"),
        pytest.param(["Q2", "Q3"], ["Q2", "Q3"], id="(col: list[2] | ignore: list[2])"),
        
    ]
)
def test_survey_quick_summary_input_types(survey, column, ignore):
    _ = survey.quick_summary(columns=column, ignore=ignore)


@pytest.mark.parametrize(
    "col1, col2",
    [
        pytest.param("Q1.a", "Q1.b", id="(original, original)"),
        pytest.param("Q1.a".lower(), "Q1.b".lower(), id="(lower, lower)"),
        pytest.param("Q1.a".upper(), "Q1.b".upper(), id="(upper, upper)"),
    ]
)
def test_survey_quick_summary_case_insensitive(survey, col1, col2):
    _ = survey.quick_summary(col1, col2)
    
    
def test_survey_crosstab_case_insensitive(survey):
    for name in survey.question_names:
        _ = survey.crosstab(name.lower(), name.lower())
        _ = survey.crosstab(name.upper(), name.upper())
        

# ============================================================================= 
# For testing crosstab-related calculations
# ============================================================================= 
@pytest.fixture
def survey_dummy():
    df = pl.DataFrame(
        {
            "QA": ["a", "b", "a", "b", "c"],
            "QB": [1, 2, 1, 3, 3],
            "QC": [[5, 4], [4], [1,2], None, [1,2,3]],
        }
    )
    
    survey_data = SurveyData.from_polars(df)
    
    dummy_codes = [
        Code.unsigned(1, "Code 1"),
        Code.unsigned(2, "Code 2"),
        Code.unsigned(3, "Code 3"),
        Code.unsigned(4, "Code 4"),
        Code.unsigned(5, "Code 5"),
    ]
    datamap = Datamap(
        name=None,
        title=None,
        version=None,
        questions=[
            Question(name="QA", question_type="character"),
            Question(name="QB", question_type="single", values=dummy_codes),
            Question(name="QC", question_type="multiple", values=dummy_codes)
        ]
    )
    
    survey = Survey(datamap, survey_data)
    return survey

def test_create_survey_from_python(survey_dummy):
    assert survey_dummy.num_response == 5


def test_value_counts_and_corss_tab_compare(survey_dummy):
    """QuestionData.value_counts() should return same result as crosstab() with columns=None
    """
    
    qname = "QC"
    value_counts = survey_dummy[qname].value_counts(normalise=False)
    value_percent = survey_dummy[qname].value_counts(normalise=True)
    
    crosstab_count = survey_dummy.crosstab(qname, normalise=False)
    crosstab_percent = survey_dummy.crosstab(qname, normalise=True)
    
    assert (value_counts.to_pandas() == crosstab_count.to_pandas()).all().all()
    assert (value_percent.to_pandas() == crosstab_percent.to_pandas()).all().all()
    
    value_counts_with_code_value = survey_dummy[qname].value_counts(normalise=False, include_response_codes=True)
    crosstab_count_with_code_value = survey_dummy.crosstab(qname, normalise=False, include_response_codes=True)
    compare = value_counts_with_code_value.to_pandas().convert_dtypes().compare(
        crosstab_count_with_code_value.to_pandas().convert_dtypes()
    )
    assert compare.empty, repr(compare)