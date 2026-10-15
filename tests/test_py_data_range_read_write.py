import pytest
from triple_s import DataRange, Response
import math


def test_create_data_range_from_new():
    with pytest.raises(NotImplementedError):
        a = DataRange(1, 2)


def test_data_range_compare():
    a = DataRange.float(2, 3)
    b = DataRange.signed(2, 3)
    c = DataRange.unsigned(2, 3)
    
    with pytest.raises(TypeError):
        assert a == b
        assert a == c
        assert b == c
    
def test_set_data_range_unsigned():
    range = DataRange.unsigned(3, 10)
    range.start = 3
    assert range.start == 3
    
    range.end = range.start
    assert range.end == 3
    
    with pytest.raises(ValueError):
        range.start = 15
        range.start = 3.5
        range.start = -3
        range.end = 0
        range.end = 3.5


def test_set_data_range_signed():
    range = DataRange.signed(3, 10)
    range.start = 3
    assert range.start == 3
    
    range.start = -3
    assert range.start == -3
    
    range.end = 55
    assert range.end == 55
    range.end = -1
    assert range.end == -1
    
    range.end = range.start
    assert range.end == -3
    
    with pytest.raises(ValueError):
        range.start = 3.5
        range.end = 3.5
        range.end = -99


def test_set_data_range_float():
    range = DataRange.float(3, 5)
    assert range.start == 3
    assert range.end == 5
    
    range.start = 2
    assert range.start == 2
    
    range.start = -1 * math.pi
    assert range.start == -1 * math.pi
        
    range.start = math.pi
    assert range.start == math.pi
    
    range.end = math.pi * 2
    assert range.end == math.pi * 2
    
    range.start = 0
    assert range.start == 0
    
    with pytest.raises(ValueError):
        range.start = math.pi *50
        range.end = -5
        
@pytest.fixture
def response():
    return Response(range=DataRange.unsigned(0, 55), values=None)
        

def test_mutate_data_range_in_response(response):
    assert response.range.start == 0
    assert response.range.end == 55
    
    response.range.start = 3
    assert response.range.start == 3
    
    range = response.range
    range.end = 7
    assert response.range.end == 7
    
    range.start = 1
    assert response.range.start == 1
    
    with pytest.raises(ValueError):
        response.range.start = 1.5
        response.range.end = -10