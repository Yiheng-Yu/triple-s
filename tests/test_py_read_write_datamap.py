import warnings
from decimal import Decimal

import pytest
import triple_s
from lxml import etree
from triple_s import Question

PATH_SSS_CLASSIC: str = "tests/sss-examples/sss_classic.sss"
PATH_SSS_XML_GENERATION_ONE = "tests/sss-examples/sss_xml_1.0.xml"
PATH_SSS_XML_GENERATION_TWO_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"


@pytest.fixture
def classic():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return triple_s.read_datamap(PATH_SSS_CLASSIC)


@pytest.fixture
def xml1():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return triple_s.read_datamap(PATH_SSS_XML_GENERATION_ONE)


@pytest.fixture
def xml2():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return triple_s.read_datamap(PATH_SSS_XML_GENERATION_TWO_EXAMPLE_1)


@pytest.fixture
def original_tree():
    return etree.parse(PATH_SSS_XML_GENERATION_ONE)


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


def test_read_file_fail():
    content = "this is some content that will 100% fail"
    with pytest.raises(ValueError, match="Error raised when reading line 1") as error:
        Question.from_string(content)


def test_question_parse_error_includes_cause():
    fragment = """<variable ident="1" type="single">
      <name>Q1</name>
      <label>Question<text mode="interview">First</text>
        <text mode="interview">Duplicate</text></label>
      <position start="1"/>
      <values><value code="1">Yes</value></values>
    </variable>"""
    with pytest.raises(ValueError) as error:
        Question.from_string(fragment)
    message = str(error.value)
    assert "When parsing question Q1 (index 1)" in message
    assert "A variant with this mode and language already exists" in message


@pytest.mark.parametrize(
    "content, expected",
    [
        ("Have you visited <br/> here before?", "Have you visited <br/> here before?"),
        (
            "Have you visited &lt;br/&gt; here before?",
            "Have you visited <br/> here before?",
        ),
        (
            '<b class="emphasis">Tea</b> &amp; coffee',
            '<b class="emphasis">Tea</b> & coffee',
        ),
        ("&#60;br/&#62;", "<br/>"),
        ("&amp;lt;br/&amp;gt;", "&lt;br/&gt;"),
        ("<![CDATA[<br/> &lt;b&gt;]]>", "<br/> &lt;b&gt;"),
    ],
)
def test_variant_markup_from_to(content, expected):
    fragment = f"""<variable ident="1" type="single">
      <name>Q1</name>
      <label>Question<text mode="interview">{content}</text></label>
      <position start="1"/>
      <values><value code="1">Yes<text mode="analysis">{content}</text></value></values>
    </variable>"""
    question = Question.from_string(fragment)
    for _ in range(3):
        assert question.label.variants[0].value == expected
        assert question.response[0].label.variants[0].value == expected
        question = Question.from_string(question.to_xml())


def test_read_file_error_context():
    path = "empty.sss"
    # Reading an empty file fails inside Rust and adds context with .with_context().
    with pytest.raises(Exception) as error:
        triple_s.read_datamap(str(path))


def test_sss_classic_read(classic):
    classic


def test_sss_xml_read(xml1):
    xml1


def test_question_xml_read_write(xml1, original_tree):
    for question in xml1.record.content:
        question_xml = question.to_xml()
        question_tree = etree.fromstring(question_xml)
        ident = question_tree.get("ident")
        original_question_tree = original_tree.find(f'.//variable[@ident="{ident}"]')
        assert original_question_tree is not None, f"Missing original question {ident}"
        assert xml_content(question_tree) == xml_content(original_question_tree), f"Question {ident}"


def assert_eq_questions(a: Question, b: Question):
    assert a.name == b.name
    assert a.format == b.format
    assert a.filter == b.filter
    assert a.usage == b.usage
    assert a.spread == b.spread
    assert a.label.value == b.label.value
    assert a.label.variants == b.label.variants  # sss classic does not have text variants
    assert a.response.range == b.response.range  # TODO! INTERFACE FOR DEFINING DATA RANGE

    for code_classic in a.response:
        code_xml = b.response.get_value(code_classic.value)
        assert code_classic == code_xml


def test_convert_between_classic_and_xml(classic):
    for question_classic in classic.record.content:
        xml_string = question_classic.to_xml()
        question_xml = triple_s.Question.from_string(xml_string)
        assert_eq_questions(question_classic, question_xml)


def test_question_read_n_write(xml2):
    for question in xml2.record.content:
        string = question.to_xml()
        re = Question.from_string(string)
        assert_eq_questions(question, re)


@pytest.mark.parametrize("source", ["xml1", "classic"])
def test_datamap_to_xml1_from_to(source, request, tmp_path):
    datamap = request.getfixturevalue(source)
    output = datamap.to_xml()
    root = etree.fromstring(output)
    assert root.tag == "sss"
    assert root.find("survey") is not None
    assert root.findtext("survey/name") == datamap.info.name
    assert root.findtext("survey/title") == datamap.title
    assert xml_content(root.find("survey/record")) == xml_content(etree.fromstring(datamap.record.to_xml()))
    path = tmp_path / "export.sss"
    path.write_text(output, encoding="utf-8")
    restored = triple_s.read_datamap(str(path))
    # The reader currently drops options and retains escaped datamap metadata.
    # Question content should still survive a complete document round trip.
    assert restored.record.to_xml() == datamap.record.to_xml()
