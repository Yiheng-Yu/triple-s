import pytest
from triple_s import DataRange, Question, QuestionType, Response


@pytest.fixture
def single():
    single = Question("Q1", "single")
    single.response = Response(DataRange.unsigned(1, 10))
    assert single.type == "single"
    return single


@pytest.fixture
def multiple():
    multiple = Question("Q1", "multiple")
    multiple.response = Response(DataRange.unsigned(1, 10))
    assert multiple.type == "multiple"
    return multiple


@pytest.fixture
def date():
    date = Question("Q1", "date")
    assert date.type == "date"
    return date


@pytest.fixture
def time():
    time = Question("Q1", "time")
    assert time.type == "time"
    return time


@pytest.fixture
def logical():
    logical = Question("Q1", "logical")
    assert logical.type == "logical"
    return logical


@pytest.fixture
def none():
    none = Question("Q1", None)
    assert none.type is None
    return none


def test_question_requires_explicit_type():
    with pytest.raises(TypeError, match="question_type"):
        Question("Q1")


@pytest.mark.parametrize("question_type", [None, "none", getattr(QuestionType, "None")])
def test_question_without_response_type(question_type):
    question = Question("n1", question_type)
    assert question.name == "n1"
    assert question.type is None


@pytest.mark.parametrize("question_type", [42, 1.5, [], {}])
def test_question_rejects_invalid_type(question_type):
    with pytest.raises(TypeError):
        Question("n1", question_type)


def test_question_set_name(single):
    single.name = "Q2"
    assert single.name == "Q2"

    with pytest.raises(TypeError):
        single.name = None
        single.name = 3
        single.name = print


@pytest.mark.parametrize(
    "target",
    [
        pytest.param("multiple", id="single-to-multiple"),
        pytest.param(None, id="single-to-none"),
    ],
)
def test_set_type_single_success(single, target):
    single.type = target
    assert single.type == target


@pytest.mark.parametrize(
    "target",
    [
        pytest.param("quantity", id="multiple-to-quantity"),
        pytest.param(None, id="multiple-to-none"),
    ],
)
def test_set_type_multiple_success(multiple, target):
    multiple.type = target
    assert multiple.type == target


@pytest.mark.parametrize(
    "target",
    [
        pytest.param("quantity", id="date-to-quantity"),
        pytest.param("character", id="date-to-character"),
        pytest.param(None, id="date-to-none"),
    ],
)
def test_set_type_date_success(date, target):
    date.type = target
    assert date.type == target


@pytest.mark.parametrize(
    "target",
    [
        pytest.param("quantity", id="time-to-quantity"),
        pytest.param("character", id="time-to-character"),
        pytest.param(None, id="time-to-none"),
    ],
)
def test_set_type_time_success(time, target):
    time.type = target
    assert time.type == target


@pytest.mark.parametrize(
    "target",
    [
        pytest.param("single", id="logical-to-single"),
        pytest.param("character", id="logical-to-character"),
        pytest.param("quantity", id="logical-to-quantity"),
        pytest.param(None, id="logical-to-none"),
    ],
)
def test_set_type_logical_success(logical, target):
    logical.type = target
    assert logical.type == target
    

@pytest.mark.parametrize(
    "target",
    [
        pytest.param("single", id="none-to-single"),
        pytest.param("character", id="none-to-character"),
        pytest.param("quantity", id="none-to-quantity"),
        pytest.param("multiple", id="none-to-multiple"),
        pytest.param("date", id="none-to-date"),
        pytest.param("time", id="none-to-time"),
        pytest.param("logical", id="none-to-logical"),
        pytest.param(None, id="none-to-none"),
    ],
)
def test_set_type_none_success(none, target):
    none.type = target
    assert none.type == target
