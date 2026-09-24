import os
from pickle import NEWFALSE
import triple_s
import pprint
import pytest
from copy import copy, deepcopy

@pytest.fixture
def sss():
    path = os.path.join(os.path.dirname(__file__), "sss-examples", "sss_classic.sss")
    survey = triple_s.read_file(path)
    return survey

def test_deep_copy_survey(sss):
    original = sss
    copied = sss.deepcopy()
    
    NEW_TITLE = "new title"
    copied.title = NEW_TITLE
    assert copied.title != original.title


def test_shallow_copy_survey(sss):
    original = sss
    copied = sss.copy()
    
    NEW_TITLE = "new title"
    copied.title = NEW_TITLE
    assert copied.title == original.title
    

def test_deep_copy_meta(sss):
    original = sss.deepcopy()
    copied = original.deepcopy()

    NEW_NAME = "NEW NAME"
    copied.meta.name = "NEW NAME"
    assert copied.meta.name == NEW_NAME
    assert original.meta.name != NEW_NAME

    ANOTHER_NAME = "Another name"
    meta = copied.meta.deepcopy()
    meta.name = ANOTHER_NAME
    assert copied.meta.name != ANOTHER_NAME
    assert copied.meta.name == NEW_NAME
    assert meta.name == ANOTHER_NAME


def test_shallow_copy_meta(sss):
    original = sss.deepcopy()
    copied = original.copy()
    
    NEW_NAME = "NEW NAME"
    copied.meta.name = "NEW NAME"
    
    assert copied.meta.name == NEW_NAME
    assert original.meta.name == NEW_NAME
    
    
# def test_shallow_copy_record(sss):
#     raise NotImplementedError

# def test_shallow_copy_question(sss):
#     raise NotImplementedError


@pytest.fixture
def pytext():
    text = triple_s.Text("Test text")
    text.add_variant("variant 1", mode="interview")
    text.add_variant("variant 2", mode="analysis")
    return text
    
def test_shallow_copy_text(pytext):
    copy = pytext
    NEW_VALUE = "new value"
    copy.value = NEW_VALUE
    assert copy.value == pytext.value
    
def test_deep_copy_text(pytext):
    original_value = deepcopy(pytext.value)
    copy = pytext.deepcopy()
    
    NEW_VALUE = "new value"
    copy.value = NEW_VALUE
    
    assert copy.value != original_value
    assert pytext.value == original_value