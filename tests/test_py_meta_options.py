import pytest

from triple_s import Meta, Mode


def test_meta_options_are_tuples():
    meta = Meta("2.0", language_options=["en-GB"], mode_options=["analysis"])
    assert meta.language_options == ("en-GB",)
    assert isinstance(meta.mode_options, tuple)
    assert meta.mode_options == (Mode.Analysis,)
    
    with pytest.raises(TypeError):
        meta.language_options[0] = "fr"
        
    with pytest.raises(TypeError):
        meta.mode_options[0] = Mode.Interview


def test_meta_setter_success():
    meta = Meta("2.0")
    meta = Meta("2.0")
    meta.name = "example"
    meta.title = "title"
    meta.standard = "standardnames"
    assert (meta.name, meta.title, meta.standard) == ("example", "title", "standardnames")

    meta.language_options = ["en", "fr"]
    assert meta.language_options == ("en", "fr")
    
    meta.language_options = ("de",)
    meta.mode_options = ["interview", Mode.Analysis, None]
    assert meta.mode_options == (Mode.Interview, Mode.Analysis, getattr(Mode, "None"))
    
    meta.mode_options = ("analysis",)
    assert meta.mode_options == (Mode.Analysis,)
    
    meta.language_options = []
    meta.mode_options = []
    assert meta.language_options == meta.mode_options == ()


@pytest.mark.parametrize(
    "field, values, error",
    [
        ("language_options", ["fr", "bad_tag"], ValueError),
        ("language_options", ["fr", 42], TypeError),
        ("mode_options", ["interview", "invalid"], ValueError),
        ("mode_options", ["interview", 42], TypeError),
    ],
)
def test_meta_setter_fail(field, values, error):
    meta = Meta("2.0", language_options=["en"], mode_options=["analysis"])
    before = getattr(meta, field)
    with pytest.raises(error):
        setattr(meta, field, values)
    assert getattr(meta, field) == before
    with pytest.raises(error):
        Meta("2.0", **{field: values})
