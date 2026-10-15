import pytest
from triple_s import Response, Code, Value, DataRange


@pytest.fixture
def response():
    return Response(
        range=DataRange.unsigned(1, 10),
        values=[
            Code.unsigned(1, "haha"),
            Code.unsigned(2, "yes", filter=None, score=3.2),
            Code.unsigned(5, None, filter="filter", score=-5.5),
        ]
    )

@pytest.fixture
def code():
    return Code.unsigned(1, "label", filter=None, score=0.2)


def test_init_response_code_value(response):
    assert response[0].code == 1
    assert response[1].code == 2
    assert response[2].code == 5
    assert response[-1].code == 5
    assert response[-2].code == 2


def test_init_response_code_label_text(response):
    assert response[0].label.value == "haha"
    assert response[1].label.value == "yes"
    assert response[2].label.value == ""
    assert response[-1].label.value == ""
    assert response[-2].label.value == "yes"


def test_init_response_filter(response):
    assert response[0].filter == None
    assert response[1].filter == None
    assert response[2].filter == "filter"
    assert response[-1].filter == "filter"
    assert response[-2].filter == None


def test_init_response_score(response):
    assert response[0].score == None
    assert response[1].score == 3.2
    assert response[2].score == -5.5
    assert response[-1].score == -5.5
    assert response[-2].score == 3.2


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
    assert response[0].code == -1


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

def test_update_code(code):
    code.label.add_variant("variant", "analysis")
    code.filter = "Q2"
    assert code.filter == "Q2"


@pytest.mark.parametrize("index, expected", [(0, 1), (1, 2), (2, 5), (-1, 5), (-2, 2), (-3, 1)])
def test_response_getitem_integer(response, index, expected):
    result = response[index]
    assert isinstance(result, Code)
    assert result.code == expected


@pytest.mark.parametrize("index", [3, -4, 100, -100])
def test_response_getitem_out_of_bounds(response, index):
    with pytest.raises(IndexError):
        response[index]


@pytest.mark.parametrize(
    "index",
    [
        slice(None), slice(1, 3), slice(None, 2), slice(1, None),
        slice(-2, None), slice(None, -1), slice(None, None, 2),
        slice(-100, 100), slice(2, 1), slice(3, 10), slice(-10, -5),
        slice(None, None, -1), slice(2, 0, -1), slice(None, None, -2),
        slice(100, -100, -1), slice(0, 2, -1),
        slice(None, None, 10**100), slice(None, None, -(10**100)),
    ],
)
def test_response_getitem_slice(response, index):
    result = response[index]
    assert isinstance(result, list)
    assert all(isinstance(item, Code) for item in result)
    assert [item.code for item in result] == [1, 2, 5][index]


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
