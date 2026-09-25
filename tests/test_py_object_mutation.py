from copy import copy, deepcopy

import pytest
from triple_s import Text, Variant, SurveyInfo, Mode, Question

# =============================================================================
# Text & Variant
# =============================================================================

VARIANT_CHANGES = [("value", "NEW"), ("mode", "interview"), ("language", "fr")]


@pytest.fixture
def variant():
    return Variant("original", mode="analysis", language="en-GB")


@pytest.fixture
def text(variant):
    return Text("original", variants=[variant])


@pytest.mark.parametrize("cls", [Text, Variant])
@pytest.mark.parametrize("value", ["NEW", "", "你好"])
def test_set_text_value(cls, value):
    obj = cls("original")
    obj.value = value
    assert obj.value == value


@pytest.mark.parametrize(
    "field, value",
    [("mode", mode) for mode in ("analysis", "interview", None)]
    + [("language", language) for language in ("fr", "en-GB", None)],
)
def test_variant_setattr_success(variant, field, value):
    setattr(variant, field, value)
    assert getattr(variant, field) == value


@pytest.mark.parametrize(
    "cls, field, value, error",
    [(cls, "value", value, TypeError) for cls in (Text, Variant) for value in (None, 42)]
    + [
        (Variant, "mode", "NEW", ValueError),
        (Variant, "mode", 42, TypeError),
        (Variant, "language", "some language", ValueError),
        (Variant, "language", 42, TypeError),
        (Text, "variants", None, TypeError),
        (Text, "variants", ["invalid"], TypeError),
    ],
)
def test_variant_setattr_fail(cls, field, value, error):
    obj = cls("original")
    before = getattr(obj, field)
    with pytest.raises(error):
        setattr(obj, field, value)
    assert getattr(obj, field) == before


@pytest.mark.parametrize("container", [list, tuple])
@pytest.mark.parametrize("empty", [False, True])
def test_replace_variants(text, variant, container, empty):
    text.variants = container([] if empty else [variant])
    assert isinstance(text.variants, tuple)
    assert len(text.variants) == (0 if empty else 1)
    if not empty:
        for field in ("value", "mode", "language"):
            assert getattr(text.variants[0], field) == getattr(variant, field)


@pytest.mark.parametrize("operation", ["construct", "append", "replace"])
@pytest.mark.parametrize("field, value", VARIANT_CHANGES)
def test_variants_are_copied_on_insertion(variant, operation, field, value):
    text = Text("original")
    if operation == "construct":
        text = Text("original", variants=[variant])
    elif operation == "append":
        text.add_variant(variant)
    else:
        text.variants = [variant]
    before = getattr(variant, field)
    setattr(variant, field, value)
    assert len(text.variants) == 1
    assert getattr(text.variants[0], field) == before


@pytest.mark.parametrize("field, value", VARIANT_CHANGES)
def test_variant_mutation_through_shared_ref(text, variant, field, value):
    shared = text.variants  # they should point to the same data
    setattr(shared[0], field, value)
    assert getattr(text.variants[0], field) == value
    assert getattr(variant, field) != value
    setattr(text.variants[0], field, getattr(variant, field))
    assert getattr(shared[0], field) == getattr(variant, field)


def test_variant_sharable_mutation(text):
    shared = text.variants
    with pytest.raises(TypeError):
        shared[0] = Variant("replacement")
    text.add_variant("another", language="fr")
    assert len(shared) == 1
    assert len(text.variants) == 2


@pytest.mark.parametrize("copy_variant", [Variant.copy, copy, deepcopy])
@pytest.mark.parametrize("field, value", VARIANT_CHANGES)
def test_variant_copy_is_independent(text, copy_variant, field, value):
    original = text.variants[0]
    copied = copy_variant(original)
    for attribute in ("value", "mode", "language"):
        assert getattr(copied, attribute) == getattr(original, attribute)
    setattr(copied, field, value)
    assert getattr(copied, field) == value
    assert getattr(original, field) != value


@pytest.mark.parametrize("as_string", [False, True])
def test_add_variant_and_reject_duplicate(text, as_string):
    variant = Variant("another", mode="interview", language="fr")
    value = variant.value if as_string else variant
    text.add_variant(value, mode=variant.mode, language=variant.language)
    assert len(text.variants) == 2
    for field in ("value", "mode", "language"):
        assert getattr(text.variants[-1], field) == getattr(variant, field)
    variant.value = "different value, same attributes"
    with pytest.raises(ValueError):
        text.add_variant(
            variant.value if as_string else variant,
            mode=variant.mode,
            language=variant.language,
        )
    assert len(text.variants) == 2
    assert text.variants[-1].value == "another"


def test_drop_duplicated_variant(variant):
    text = Text("example")
    assert len(text.variants) == 0

    with_dupe = [Variant("new", language="fr"), variant, variant.copy()]
    text.variants = with_dupe
    assert len(text.variants) == len(set(with_dupe))


@pytest.mark.parametrize("index", [0, -1])
def test_remove_variant(text, index):
    removed = text.remove_variant(index)
    assert (removed.value, removed.mode, removed.language) == ("original", "analysis", "en-GB")
    assert text.variants == ()


@pytest.mark.parametrize("index", [1, -2])
def test_remove_variant_out_of_range(text, index):
    with pytest.raises(IndexError):
        text.remove_variant(index)
    assert len(text.variants) == 1


# =============================================================================
# Meta
# =============================================================================


def test_meta_options_are_tuples():
    meta = SurveyInfo("2.0", language_options=["en-GB"], mode_options=["analysis"])
    assert meta.language_options == ("en-GB",)
    assert isinstance(meta.mode_options, tuple)
    assert meta.mode_options == (Mode.Analysis,)

    with pytest.raises(TypeError):
        meta.language_options[0] = "fr"

    with pytest.raises(TypeError):
        meta.mode_options[0] = Mode.Interview


def test_meta_setter_success():
    meta = SurveyInfo("2.0")
    meta = SurveyInfo("2.0")
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
    meta = SurveyInfo("2.0", language_options=["en"], mode_options=["analysis"])
    before = getattr(meta, field)
    with pytest.raises(error):
        setattr(meta, field, values)
    assert getattr(meta, field) == before
    with pytest.raises(error):
        SurveyInfo("2.0", **{field: values})


# =============================================================================
# Question
# =============================================================================
def create_empty_question():
    return Question("Q1")


@pytest.fixture
def question():
    return create_empty_question()

@pytest.mark.parametrize(
    "attr, value",
    [
        pytest.param(
            "label",
            Text("this is an example label"),
            id="label-value-some",
        ),
        pytest.param(
            "label",
            None,
            id="label-value-none",
        ),
        pytest.param(
            "filter",
            "some filter",
            id="filter-value-some",
        ),
        pytest.param(
            "filter",
            None,
            id="filter-value-none",
        ),
    ],
)
def test_question_setter_success(attr, value):
    example = create_empty_question()
    print(example)
    setattr(example, attr, value)

    attr_example = getattr(example, attr, None)
    if attr_example is None:
        assert value is None
    else:
        assert attr_example == value


def test_question_variant_setter_success(question):
    question.label = "this label"
    assert question.label.value == "this label"
    print(question.label)
    question.label.add_variant("this variant", mode="analysis")
    print(question.label)
    assert question.label.variants[0].value == "this variant"
    assert question.label.variants[0].mode == "analysis"


@pytest.mark.parametrize(
    "attr, value",
    [
        pytest.param(
            "label",
            3.2,
            id="label-value-int",
        ),
        pytest.param(
            "label",
            55,
            id="label-value-float",
        ),
    ],
)
def test_question_setter_fail(attr, value):
    example = create_empty_question()
    with pytest.raises(TypeError):
        setattr(example, attr, value)
