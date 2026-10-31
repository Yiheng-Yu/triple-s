import warnings

import pytest

from triple_s import SurveyData, read_datamap


@pytest.mark.parametrize("example, extension", [(1, "asc"), (2, "csv")])
def test_validate_then_load_data(example, extension):
    base = f"tests/sss-examples/sss-2.0/example{example}"
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        datamap = read_datamap(f"{base}.sss")
    record = datamap.record
    href = record.href
    data_format = record.data_format
    assert record.validate_data(f"{base}.{extension}") is None
    assert record.href == href
    assert record.data_format == data_format

    data = record.load_data(f"{base}.{extension}")
    assert isinstance(data, SurveyData)
    assert data.has_data()
    assert record.href == f"{base}.{extension}"
    assert data.to_polars().height == 3
