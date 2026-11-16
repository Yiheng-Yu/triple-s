import pytest
from triple_s import DataRange, Question, QuestionType, Response


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


def test_question_set_name():
    question = Question("Q1", "single")
    question.name = "Q2"
    assert question.name == "Q2"


@pytest.mark.parametrize("name", [None, 3, print])
def test_question_name_rejects_invalid_values(name):
    question = Question("Q1", "single")
    with pytest.raises(TypeError):
        question.name = name
    assert question.name == "Q1"


@pytest.mark.parametrize("source, target", [
    (source, target)
    for source, targets in [
        ("single", ["multiple", None]),
        ("multiple", ["quantity", None]),
        ("date", ["quantity", "character", None]),
        ("time", ["quantity", "character", None]),
        ("logical", ["single", "character", "quantity", None]),
        (None, ["single", "character", "quantity", "multiple", "date", "time",
                "logical", None]),
    ]
    for target in targets
])
def test_question_type_conversion(source, target):
    question = Question("Q1", source)
    if source in ("single", "multiple"):
        question.response = Response(DataRange.unsigned(1, 10))
    assert question.type == source
    question.type = target
    assert question.type == target
