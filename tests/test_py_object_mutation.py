import pytest
import triple_s
from triple_s import (
    Text,
    Variant,
)


@pytest.fixture
def text():
    return Text("empty text with no variants")


@pytest.fixture
def variant_a():
    return Variant("variant a", language="gb")


@pytest.fixture
def variant_b():
    return Variant("variant another", language="us")


@pytest.fixture
def variant_c():
    return Variant("variant another", language="es", mode="analysis")


def test_modify_text_value(text):
    text.value = "NEW"
    assert text.value == "NEW"


def test_modify_mode_failure(variant_b):
    with pytest.raises(ValueError):
        variant_b.mode = "NEW"


def test_modify_language_failure(variant_a):
    with pytest.raises(ValueError):
        variant_a.language = "some language"


def test_insert_variant(text, variant_a):
    before = len(text.variants)

    text.add_variant(variant_a)
    assert len(text.variants) == before + 1
    assert text.variants[-1].value == "variant a"

    variant_a.value = "haha"
    assert text.variants[0].value != variant_a.value, "should not be able to modify values that gets appended to"


def test_no_add_duplicated_variant(text, variant_a, variant_c):
    text.add_variant(variant_a)

    variant_b = variant_a.copy()
    variant_b.value = "yes"
    with pytest.raises(ValueError):
        text.add_variant(variant_b)
    
    text.add_variant(variant_c)
    with pytest.raises(ValueError):
        text.add_variant(
            "random_text", 
            mode=variant_c.mode, 
            language=variant_c.language
            ) 


def test_variant_append(text, variant_a, variant_b, variant_c):
    text.add_variant(variant_a)
    text.add_variant(variant_b)
    text.add_variant(variant_c)

    assert len(text.variants) == 3
    variant_b.mode = "interview"
    assert text.variants[1].mode != "interview", "appending an item should deep copy it"

    vlist = text.variants
    vlist[0].language = "zh"
    assert variant_a.language != "zh"
    assert vlist[0].language == "zh"

    assert variant_c.mode == "analysis"
    vlist[2].mode = None
    assert vlist[2].mode is None
    assert variant_c.mode is not None


def test_variant_deepcopy(text):
    for (idx, var) in enumerate(text.variants):
        tmp = var.copy()
        tmp.value = "tmp"
        tmp.language = "ar"
        assert text.variants[idx].value != tmp.value


def test_variant_mutation_from_pointer(text, variant_c):
    text.add_variant(variant_c)
    tmp = text.variants[-1]

    text.variants[-1].value = "what why"
    assert text.variants[-1].value == tmp.value

    tmp.mode = "interview"
    assert text.variants[-1].mode == "interview"
