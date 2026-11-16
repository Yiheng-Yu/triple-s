import pytest
from triple_s import Response, Code, DataRange, Text


@pytest.fixture
def response():
    return Response(
        range=DataRange.unsigned(1, 10),
        values=[
            Code.unsigned(1, "haha"),
            Code.unsigned(2, "yes", filter=None, score=3.2),
            Code.unsigned(5, None, filter="filter", score=-5.5),
        ],
    )


@pytest.mark.parametrize("index, label, filter, score", [
    (0, "haha", None, None),
    (1, "yes", None, 3.2),
    (2, "", "filter", -5.5),
])
def test_response_initial_code_metadata(response, index, label, filter, score):
    code = response[index]
    assert code.label.value == label
    assert code.filter == filter
    assert code.score == score


def test_update_code_value_via_indexing(response):
    response[0].label = "what"
    assert response[0].label.value == "what"

    response[0].label.add_variant("new", mode="interview")
    assert response[0].label.variants[0].value == "new"

    response[1].label.add_variant("new", mode="interview")
    assert response[1].label.variants[0].value == "new"


CODE_FACTORIES = [Code.unsigned, Code.signed, Code.float, Code.string]


@pytest.mark.parametrize("method", ["add_value", "set_value"])
@pytest.mark.parametrize("existing_factory", CODE_FACTORIES)
@pytest.mark.parametrize("incoming_factory", CODE_FACTORIES)
def test_code_variant_validation(method, existing_factory, incoming_factory):
    def make(factory, value, label):
        return factory(str(value) if factory == Code.string else value, label)

    response = Response(values=[make(existing_factory, 1, "original")])
    incoming = make(incoming_factory, 2, "new")
    if existing_factory == incoming_factory:
        getattr(response, method)(incoming)
        assert len(response.values) == 2
        assert response[1].label.value == "new"
    else:
        with pytest.raises(ValueError):
            getattr(response, method)(incoming)
        assert len(response.values) == 1
        assert response[0].label.value == "original"


@pytest.mark.parametrize("method", ["add_value", "set_value"])
def test_empty_response_accepts_first_code(method):
    response = Response()
    getattr(response, method)(Code.signed(-1, "first"))
    assert response[0].value == -1


def test_add_duplicate_and_set_existing_code(response):
    existing = response[0]
    with pytest.raises(ValueError, match="already exists"):
        response.add_value(Code.unsigned(1, "duplicate"))
    response.set_value(Code.unsigned(1, "updated", filter="filter", score=2.5))
    assert len(response.values) == 3
    assert existing.label.value == "updated"
    assert existing.filter == "filter"
    assert existing.score == 2.5
    response.set_value(existing)
    assert existing.label.value == "updated"


@pytest.mark.parametrize(
    "index, expected", [(0, 1), (1, 2), (2, 5), (-1, 5), (-2, 2), (-3, 1)]
)
def test_response_getitem_integer(response, index, expected):
    result = response[index]
    assert isinstance(result, Code)
    assert result.value == expected


@pytest.mark.parametrize("index", [3, -4, 100, -100])
def test_response_getitem_out_of_bounds(response, index):
    with pytest.raises(IndexError):
        response[index]


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
        slice(-100, 100),
        slice(2, 1),
        slice(3, 10),
        slice(-10, -5),
        slice(None, None, -1),
        slice(2, 0, -1),
        slice(None, None, -2),
        slice(100, -100, -1),
        slice(0, 2, -1),
        slice(None, None, 10**100),
        slice(None, None, -(10**100)),
    ],
)
def test_response_getitem_slice(response, index):
    result = response[index]
    assert isinstance(result, list)
    assert all(isinstance(item, Code) for item in result)
    assert [item.value for item in result] == [1, 2, 5][index]


@pytest.mark.parametrize("index", ["1", 1.5, None, [0], (0,), {}])
def test_response_getitem_invalid_type(response, index):
    with pytest.raises(TypeError):
        response[index]


def test_response_getitem_zero_slice_step(response):
    with pytest.raises(ValueError):
        response[::0]


@pytest.mark.parametrize("index", [0, -1])
def test_empty_response_getitem_integer(index):
    with pytest.raises(IndexError):
        Response()[index]


@pytest.mark.parametrize("index", [slice(None), slice(None, None, -1), slice(0, 10)])
def test_empty_response_getitem_slice(index):
    assert Response()[index] == []


def test_response_getitem_slice_shares_codes(response):
    result = response[1:]
    result[0].label = "updated through slice"
    assert response[1].label.value == "updated through slice"
    response[2].score = 4.5
    assert result[1].score == 4.5
    result.pop()
    assert len(response.values) == 3


@pytest.mark.parametrize("factory, value", [
    (Code.unsigned, 2), (Code.signed, -2), (Code.float, 2.5), (Code.string, "A"),
])
def test_code_factory_fields_and_mutation(factory, value):
    code = factory(value, label="Label", filter="Q1", score=1.5)
    assert code.value == value
    assert code.label == Text("Label")
    assert code.filter == "Q1"
    assert code.score == 1.5
    code.filter = "Updated filter"
    assert code.filter == "Updated filter"
    code.score = -2.5
    assert code.score == -2.5
    code.score = code.filter = None
    assert code.score is code.filter is None
    code.label = Text("Updated")
    assert code.label.value == "Updated"
    default = factory(value)
    assert default.label == Text("")
    assert default.filter is default.score is None


def test_code_direct_construction_is_unsupported():
    with pytest.raises(NotImplementedError):
        Code(1, None, None, None)


@pytest.mark.parametrize("factory, value, lookup, missing", [
    (Code.unsigned, 2, 2.0, "2"),
    (Code.signed, -2, -2.0, "-2"),
    (Code.float, 2.0, 2, "2"),
    (Code.string, "2", "2", 2),
])
def test_response_get_value_lookup_and_shared_mutation(factory, value, lookup, missing):
    response = Response(values=[factory(value, "Original")])
    found = response.get_value(lookup)
    assert found.value == value
    assert response.get_value(missing) is None
    found.label = "Updated"
    assert response[0].label.value == "Updated"
    assert Response().get_value(lookup) is None


@pytest.mark.parametrize("value", [None, [], {}, object()])
def test_response_get_value_rejects_invalid_types(response, value):
    with pytest.raises(TypeError, match="str, int, or float"):
        response.get_value(value)


@pytest.mark.parametrize("as_objects", [False, True])
def test_response_from_dict_range_and_mixed_codes(as_objects):
    bounds = DataRange.unsigned(0, 5) if as_objects else {"start": 0, "end": 5}
    response = Response.from_dict({
        "range": bounds,
        "values": [Code.unsigned(1, "One"), {"value": 2, "label": "Two"}],
    })
    assert response.range == DataRange.unsigned(0, 5)
    assert isinstance(response.values, tuple)
    assert [code.value for code in response.values] == [1, 2]
    assert response.response_label == {1: Text("One"), 2: Text("Two")}
    empty = Response.from_dict({})
    assert empty.range is None
    assert empty.values == ()
    assert empty.response_label == {}


@pytest.mark.parametrize("content", [{"range": "invalid"}, {"values": None},
                                      {"values": [42]}])
def test_response_from_dict_rejects_invalid_children(content):
    with pytest.raises(TypeError):
        Response.from_dict(content)


def test_response_label_map_returns_independent_text(response):
    labels = response.response_label
    labels[1].value = "Changed copy"
    labels.pop(2)
    assert response[0].label.value == "haha"
    assert response.response_label[2].value == "yes"
