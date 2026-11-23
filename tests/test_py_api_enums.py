from triple_s import Version
import pytest

@pytest.mark.parametrize(
    "value",
    [
        pytest.param(1, id="int"),
        pytest.param(1.2, id="float1"),
        pytest.param(2.0, id="float2"),
        pytest.param("2.0", id="str"),
        pytest.param("1.2", id="str"),
    ]
)
def test_create_version_instance_success(value):
    _ = Version.new(value)
    _ = Version(value)


def test_create_version_instance_fail():
    with pytest.raises(TypeError):
        Version.new(dict())
        Version(dict())
        
        Version.new(lambda s: s)
        Version(lambda s: s)

        Version.new(list())
        Version(list())
        
    with pytest.raises(ValueError):
        Version.new(42)
        Version(42)
    
        