import os
from pickle import NEWFALSE
import triple_s
import pprint
import pytest

@pytest.fixture
def sss():
    path = os.path.join(os.path.dirname(__file__), "sss-examples", "sss_classic.sss")
    survey = triple_s.triple_s.read_file(path)
    return survey

def test_deep_copy_meta(sss):
    original = sss.copy()
    copied = original.copy()
    print("Before editing:")
    print(f"Original: {original.meta.name}")
    print(f"Copy: {copied.meta.name}")
    
    NEW_NAME = "NEW NAME"
    copied.meta.name = "NEW NAME"
    assert copied.meta.name == NEW_NAME
    assert original.meta.name != NEW_NAME
    
    ANOTHER_NAME = "Another name"
    meta = copied.meta.copy()
    meta.name = ANOTHER_NAME
    assert copied.meta.name != ANOTHER_NAME
    assert copied.meta.name == NEW_NAME
    assert meta.name == ANOTHER_NAME