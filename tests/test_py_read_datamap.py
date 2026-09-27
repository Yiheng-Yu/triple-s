from copy import copy, deepcopy

import pytest
import warnings

import triple_s
from triple_s import Text, Variant, SurveyInfo, Question
from triple_s.flags import Mode, CodeFormat, Usage

PATH_SSS_CLASSIC: str = "tests/sss-examples/sss_classic.sss"
PATH_DATAMAP_XML = "tests/sss-examples/sss_xml_1.0.xml"


@pytest.fixture
def classic():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return triple_s.read_datamap(PATH_SSS_CLASSIC)


@pytest.fixture
def xml():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return triple_s.read_datamap(PATH_DATAMAP_XML)


def test_sss_classic_read(classic):
    print()
    for question in classic.record.content:
        print(question)


def test_sss_xml_read(xml):
    print()
    for question in xml.record.content:
        print(question)
