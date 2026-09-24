import pytest
import triple_s
import os
from pprint import pprint
from copy import deepcopy

def test_create_meta_all_fields():
    fields = {
        "version": "1.2",
        "name": "survey_name",
        "title": "Survey title",
        "date": "2026-09-24",
        "time": "14:30:00",
        "origin": "Survey application",
        "user": "Survey author",
        "style": "custom",
        "language_options": ["en-gb", "fr"],
        "mode_options": [None, "analysis", "interview"],
        "standard": "standardnames",
    }
    meta = triple_s.Meta(**fields)
    for field, expected in fields.items():
        if isinstance(expected, list):
            expected = tuple(expected)
        assert getattr(meta, field) == expected, field

    defaults = triple_s.Meta("1.2")
    for field in fields:
        expected = {
            "version": "1.2",
            "language_options": (),
            "mode_options": (None,),
        }.get(field)
        assert getattr(defaults, field) == expected, field

    empty = triple_s.Meta("1.2", language_options=[], mode_options=[])
    assert empty.language_options == ()
    assert empty.mode_options == ()

    with pytest.raises(ValueError):
        triple_s.Meta("invalid")
    with pytest.raises(ValueError):
        triple_s.Meta("1.2", mode_options=["invalid"])

@pytest.fixture
def survey():
    path = os.path.join(os.path.dirname(__file__), "sss-examples", "sss_classic.sss")
    survey = triple_s.read_file(path)
    return survey

@pytest.fixture
def question(survey):
    return survey.record.content[0]

@pytest.fixture
def variant_a():
    variant_a = triple_s.Variant("test", mode="interview", language="GB")
    assert variant_a.value == "test"
    assert variant_a.mode == "interview"
    return variant_a

def test_set_variant_language_success(variant_a):
    variant_a.language = "en-gb"
    assert variant_a.language == "en-gb", variant_a.language

def test_set_variant_mode_success(variant_a):
    variant_a.mode = None
    assert variant_a.mode is None
    
    variant_a.mode = "interview"
    assert variant_a.mode == "interview", variant_a.mode
    
    variant_a.mode = "analysis"
    assert variant_a.mode == "analysis", variant_a.mode
    
def test_set_variant_attributes_fail(variant_a):
    with pytest.raises(TypeError):
        variant_a.value = None
        
    with pytest.raises(ValueError):
        variant_a.mode = "some mode"
        variant_a.language = "this is not an accepted language value"

@pytest.fixture
def pytext():
    text = triple_s.Text("Test text")
    text.add_variant("variant 1", mode="interview")
    text.add_variant("variant 2", language="en-gb")
    text.add_variant("variant 3", mode="analysis", language=None)
    return text

def test_add_variant_error(pytext):
    with pytest.raises(ValueError):
        pytext.add_variant("variant 2", mode="interview")
        pytext.add_variant("variant 2", mode=None, language=None)
        pytext.add_variant("variant 2")

def test_modify_text(pytext):
    NEW_VALUE = "this is new value"
    pytext.value = NEW_VALUE
    assert pytext.value == NEW_VALUE
    
    pytext.variants[0].value = NEW_VALUE
    assert pytext.variants[0].value == NEW_VALUE, print(pytext.variants[0])
    
    
def test_modify_text_variant_list(pytext):
    another_one = triple_s.Variant("this is another one", mode=None, language="tr")
    before = len(pytext.variants)
    pytext.variants.append(another_one)
    assert len(pytext.variants) == ( before + 1 )
    
    poped = pytext.variants.pop()
    assert len(pytext.variants) == before
    assert poped.value == "this is another one"
    
    poped.value = "yet another new title"
    assert pytext.variants[-1].value != poped.value
    
    
def test_modify_question_label(question):
    NEW_LABEL = "THIS IS A VERY VERY VERY VERY NEW LABEL"
    question.label = NEW_LABEL
    assert question.label.value == NEW_LABEL
    
    
def test_modify_question_label_via_survey_indexing(survey):
    NEW_LABEL = "THIS IS A VERY VERY VERY VERY NEW LABEL"
    original_label = deepcopy(survey.record.content[3].label.value)
    question = survey.record.content[3]
    question.label = NEW_LABEL
    assert survey.record.content[3].label.value == NEW_LABEL
    assert survey.record.content[3].label.value != original_label

    tmp = question.label
    TMP_LABEL = "label for tmp"
    tmp.value = TMP_LABEL
    assert tmp.value == TMP_LABEL
    assert question.label.value == TMP_LABEL
    assert survey.record.content[3].label.value == TMP_LABEL

def test_insert_label_variant_via_survey_indexing(survey):
    VARIANT_TEXT = "text"
    VARIANT_LANGUAGE = "es"
    VARIANT_MODE = "interview"
    INDEX = 5
    variant = triple_s.Variant(VARIANT_TEXT, mode=VARIANT_MODE, language=VARIANT_LANGUAGE)

    len_original = len(survey.record.content[INDEX].label.variants)
    question = survey.record.content[INDEX]
    variants = question.label.variants

    # question.label.variants.append(variant)
    variants.append(variant)
    assert len(variants) == (len_original + 1)
    assert len(question.label.variants) == (len_original + 1)
    assert len(survey.record.content[INDEX].label.variants) == (len_original + 1)

    NEW_LABEL = "new label"
    variant.value = NEW_LABEL
    assert variants[-1].value == NEW_LABEL
    assert question.variants[-1].value == NEW_LABEL
    assert survey.record.content[INDEX].label.variants[-1].value == NEW_LABEL
    


def test_modify_question_location(question):
    question.location.start = 300
    assert question.location.start == 300

    tmp = quesiton.location
    assert tmp.location.start == 300
    tmp.location.end = 999
    assert question.location.end == 999

# def test_create_question():
#     raise NotImplementedError


# def test_modify_question_name_success(sss):
#     raise NotImplementedError


# def test_modify_question_name_fail(sss):
#     # should fail if the `Record` instance that contains current instance already has a question of the same name.
#     raise NotImplementedError
