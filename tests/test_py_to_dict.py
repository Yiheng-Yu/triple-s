import pytest
from triple_s import Variant, Text


# =============================================================================
# Variants
# =============================================================================

@pytest.fixture
def dict_variant_1() -> dict:
    return dict(value="test value", mode="interview", language="tr")


@pytest.fixture
def dict_variant_2() -> dict:
    return dict(value="test value 2", mode="analysis", language=None)


@pytest.fixture
def dict_variant_3() -> dict:
    return dict(value="test value 3", language="en-gb")


@pytest.mark.parametrize(
    "dictionary",
    [
        pytest.param("dict_variant_1", id="variant-1"),
        pytest.param("dict_variant_2", id="variant-2"),
        pytest.param("dict_variant_3", id="variant-3"),
    ],
)
def test_variant_to_dict(dictionary, request):
    dictionary = request.getfixturevalue(dictionary)
    
    instance = Variant.from_dict(dictionary)
    output = instance.to_dict()

    for key_output, value_output in output.items():
        assert value_output == dictionary.get(key_output)


# =============================================================================
# Texts
# =============================================================================
@pytest.fixture
def dict_text_1():
    return dict(
        value="Test text value 1",
        variants=[
            dict(value="test value", mode="interview", language="tr"),
            dict(value="test value 2", mode="analysis", language=None)
        ],
    )


def test_text_to_dict(dict_text_1):
    dictionary = dict_text_1
    instance = Text.from_dict(dictionary)
    output = instance.to_dict()
    new_instance = Text.from_dict(output)
    assert new_instance == instance
