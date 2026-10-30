import warnings

import pytest

from triple_s import NoDataError, read_datamap


@pytest.mark.parametrize("example, extension", [(1, "asc"), (2, "csv")])
def test_validate_then_load_data(example, extension):
    base = f"tests/sss-examples/sss-2.0/example{example}"
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        datamap = read_datamap(f"{base}.sss")
    record = datamap.record
    href = record.href
    record.validate_data(f"{base}.{extension}")
    assert record.href == href
    with pytest.raises(NoDataError):
        record.data.to_polars()

    record.load_data(f"{base}.{extension}")
    assert record.href == f"{base}.{extension}"
    assert datamap.data.to_polars().height == 3
