import math

import pytest
from triple_s import DataRange


def test_create_data_range_from_new():
    with pytest.raises(NotImplementedError):
        DataRange(1, 2)


def test_data_range_compare():
    unsigned = DataRange.unsigned(2, 3)
    signed = DataRange.signed(2, 3)
    floating = DataRange.float(2, 3)
    assert floating != signed
    assert floating != unsigned
    assert signed == unsigned


@pytest.mark.parametrize("factory, bounds, updates", [
    (DataRange.unsigned, (3, 10), [("start", 3), ("end", 3)]),
    (DataRange.signed, (3, 10), [
        ("start", 3), ("start", -3), ("end", 55), ("end", -1), ("end", -3),
    ]),
    (DataRange.float, (3, 5), [
        ("start", 2), ("start", -math.pi), ("start", math.pi),
        ("end", math.pi * 2), ("start", 0),
    ]),
])
def test_set_data_range(factory, bounds, updates):
    data_range = factory(*bounds)
    assert (data_range.start, data_range.end) == bounds
    for field, value in updates:
        setattr(data_range, field, value)
        assert getattr(data_range, field) == value


@pytest.mark.parametrize("factory, field, value", [
    (DataRange.unsigned, "start", 15),
    (DataRange.unsigned, "start", 3.5),
    (DataRange.unsigned, "start", -3),
    (DataRange.unsigned, "end", 0),
    (DataRange.unsigned, "end", 3.5),
    (DataRange.signed, "start", 3.5),
    (DataRange.signed, "end", 3.5),
    (DataRange.signed, "end", -99),
    (DataRange.float, "start", math.pi * 50),
    (DataRange.float, "end", -5),
])
def test_data_range_setters_reject_each_invalid_bound(factory, field, value):
    data_range = factory(3, 10)
    with pytest.raises(ValueError):
        setattr(data_range, field, value)
    assert (data_range.start, data_range.end) == (3, 10)
