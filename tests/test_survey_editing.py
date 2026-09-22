import os
import triple_s
import pprint
import pytest

@pytest.fixture
def sss():
    path = os.path.join(os.path.dirname(__file__), "sss-examples", "sss_classic.sss")
    survey = triple_s.triple_s.read_file(path)
    return survey

def test_deep_copy(sss):
    original = sss.copy()
    copied = original.copy()
    
    NEW_NAME = "NEW NAME"
    copied.meta.name = "NEW NAME"
    assert copied.meta.name == NEW_NAME
    assert original.meta.name != NEW_NAME