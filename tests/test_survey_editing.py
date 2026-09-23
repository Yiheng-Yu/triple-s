import os
from pickle import NEWFALSE
import triple_s
import pprint
import pytest


@pytest.fixture
def sss():
    path = os.path.join(os.path.dirname(__file__), "sss-examples", "sss_classic.sss")
    survey = triple_s.read_file(path)
    return survey


def test_shallow_copy_meta(sss):
    raise NotImplementedError


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
    with pytest.raises(AssertionError):
        assert copied.meta.name == NEW_NAME
        assert original.meta.name != NEW_NAME

def test_deep_copy_record(sss):
    raise NotImplementedError


def test_shallow_copy_record(sss):
    raise NotImplementedError


def test_deep_copy_question(sss):
    raise NotImplementedError


def test_shallow_copy_question(sss):
    raise NotImplementedError

# todo: set up another buch of unit tests!


def test_create_question(sss):
    raise NotImplementedError


def test_modify_question_label(sss):
    raise NotImplementedError


def test_modify_question_name_success(sss):
    raise NotImplementedError


def test_modify_question_name_fail(sss):
    # should fail if the `Record` instance that contains current instance already has a question of the same name.
    raise NotImplementedError


def test_create_and_modify_text(sss):
    raise NotImplementedError


def test_modify_variants_inside_text(sss):
    raise NotImplementedError


def test_modify_text_variant(sss):
    raise NotImplementedError
