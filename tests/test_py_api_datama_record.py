import warnings
from decimal import Decimal

import pytest
import triple_s
from lxml import etree
from triple_s import Question

PATH_SSS_XML_GENERATION_ONE = "tests/sss-examples/sss_xml_1.0.xml"


@pytest.fixture
def record():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return triple_s.read_datamap(PATH_SSS_XML_GENERATION_ONE)


def xml_content(tree):
    """Used for comparing xml1 elements to see if they were same or not,,quick and dirty"""
    content = []
    for element in tree.iter():
        if not isinstance(element.tag, str) or element.tag == "values":
            continue
        attributes = dict(element.attrib)
        if element.tag == "range":
            attributes = {key: Decimal(value) for key, value in attributes.items()}
        content.append((element.tag, sorted(attributes.items()), (element.text or "").strip()))
    return sorted(content)


def test_record_slice_indexing(record):
    result = record[0:3]
    assert len(result) == 3
    result = record[0:-2]
    assert len(result) == (len(record.content) - 2)


@pytest.mark.parametrize("index", [0, 1, -1, -2])
def test_record_getitem_integer(record, index):
    result = record[index]
    assert isinstance(result, Question)
    assert result.name == record.content[index].name


def test_record_getitem_integer_boundaries(record):
    assert record[-len(record)].name == record[0].name
    for index in [len(record), -len(record) - 1, 10000, -10000]:
        with pytest.raises(IndexError):
            record[index]


@pytest.mark.parametrize("case", [str, str.lower, str.upper])
def test_record_getitem_name(record, case):
    expected = record.content[1]
    result = record[case(expected.name)]
    assert isinstance(result, Question)
    assert result.name == expected.name


@pytest.mark.parametrize("name", ["missing_question", ""])
def test_record_getitem_missing_name(record, name):
    with pytest.raises(KeyError):
        record[name]


@pytest.mark.parametrize(
    "index",
    [
        slice(None),
        slice(1, 3),
        slice(None, 2),
        slice(1, None),
        slice(-2, None),
        slice(None, -1),
        slice(None, None, 2),
        slice(-10000, 10000),
        slice(2, 1),
        slice(10000, 10001),
        slice(-10001, -10000),
        slice(None, None, -1),
        slice(2, 0, -1),
        slice(None, None, -2),
        slice(10000, -10000, -1),
        slice(0, 2, -1),
        slice(None, None, 10**100),
        slice(None, None, -(10**100)),
    ],
)
def test_record_getitem_slice(record, index):
    result = record[index]
    assert isinstance(result, list)
    assert all(isinstance(item, Question) for item in result)
    expected = record.content[index]
    assert [item.name for item in result] == [item.name for item in expected]


@pytest.mark.parametrize("index", [1.5, None, [0], (0,), {}])
def test_record_getitem_invalid_type(record, index):
    with pytest.raises(TypeError):
        record[index]


def test_record_getitem_zero_slice_step(record):
    with pytest.raises(ValueError):
        record[::0]


@pytest.mark.parametrize("lookup", ["integer", "name", "slice"])
def test_record_getitem_shares_question(record, lookup):
    record = record
    if lookup == "integer":
        result = record[0]
    elif lookup == "name":
        result = record[record.content[0].name]
    else:
        result = record[:1][0]
    result.label = "updated through indexing"
    assert record.content[0].label.value == "updated through indexing"


def test_record_getitem_slice_has_independent_container(record):
    original_length = len(record)
    result = record[:]
    result.pop()
    assert len(record) == original_length


def test_record_to_xml(record):
    record.href = 'data & "answers".csv'
    record.data_format = "csv"
    root = etree.fromstring(record.to_xml())
    assert root.tag == "record"
    assert root.attrib == {
        "ident": record.ident,
        "href": record.href,
        "format": "csv",
    }
    assert [item.findtext("name") for item in root] == [question.name for question in record.content]
    for element, question in zip(root, record.content):
        assert xml_content(element) == xml_content(etree.fromstring(question.to_xml()))
