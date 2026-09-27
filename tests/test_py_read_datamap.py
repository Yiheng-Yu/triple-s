from copy import copy, deepcopy

import pytest

import triple_s
from triple_s import Text, Variant, SurveyInfo, Question
from triple_s.flags import Mode, CodeFormat, Usage

PATH_SSS_CLASSIC: str = "tests/sss-examples/sss_classic.sss"

@pytest.fixture
def classic():
    return triple_s.read_datamap(PATH_SSS_CLASSIC)
    
def test_sss_classic_read(classic):
    print(classic)