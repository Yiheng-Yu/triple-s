from triple_s import Variant

def test_variant_to_dict():
    content = dict(
        value = "test value",
        mode = "interview",
        language = "tr"
    )
    variant = Variant.from_dict(content)
    assert variant.to_dict() == content
    