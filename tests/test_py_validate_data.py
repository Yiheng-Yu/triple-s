import warnings
from pathlib import Path

import pytest

from triple_s import Location, Question, Record, SurveyData, read_datamap


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


@pytest.mark.parametrize("example, extension", [(1, "asc"), (2, "csv")])
def test_invalid_utf8_is_replaced(tmp_path, example, extension):
    base = f"tests/sss-examples/sss-2.0/example{example}"
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        datamap = read_datamap(f"{base}.sss")
    path = tmp_path / f"responses.{extension}"
    path.write_bytes(
        Path(f"{base}.{extension}")
        .read_bytes()
        .replace(b"Nottingham Goose Fair", b"\x96ottingham Goose Fair")
    )

    assert datamap.record.validate_data(str(path)) is None
    data = datamap.load_data(str(path))
    frame = data.to_polars()
    assert frame.height == 3
    assert frame["Q3.a"][0].rstrip() == "�ottingham Goose Fair"
    assert frame["Q4"].to_list() == [2, 9, 1]


@pytest.fixture
def text_record():
    return Record.from_dict(
        {"content": [Question("TEXT", "character", location=Location(1, 6))]}
    )


@pytest.mark.parametrize("extension", ["asc", "csv"])
@pytest.mark.parametrize(
    "encoding,bom",
    [
        ("utf-8", b""),
        ("utf-8-sig", b""),
        ("utf-16-le", b"\xff\xfe"),
        ("utf-16-be", b"\xfe\xff"),
    ],
)
def test_valid_unicode_encodings_are_preserved(tmp_path, text_record, extension, encoding, bom):
    path = tmp_path / f"responses.{extension}"
    path.write_bytes(bom + "Café—😀\n".encode(encoding))

    assert text_record.validate_data(str(path)) is None
    data = text_record.load_data(str(path))
    assert data.to_polars()["TEXT"].to_list() == ["Café—😀"]


def test_fixed_width_replacement_preserves_records_and_truncated_eof(tmp_path, text_record):
    path = tmp_path / "responses.asc"
    path.write_bytes(b"\x96ab\xff\x80c\n" + "Café—😀\r\n".encode() + b"abcde\xe2\x82")

    assert text_record.validate_data(str(path)) is None
    data = text_record.load_data(str(path))
    assert data.to_polars()["TEXT"].to_list() == ["�ab��c", "Café—😀", "abcde�"]


def test_csv_replacement_preserves_quoting_and_multiline_fields(tmp_path, text_record):
    path = tmp_path / "responses.csv"
    path.write_bytes(b'"\x96say, ""hi""\nsecond\xff",7\r\n')

    assert text_record.validate_data(str(path)) is None
    data = text_record.load_data(str(path))
    assert data.to_polars()["TEXT"].to_list() == ['�say, "hi"\nsecond�']


@pytest.mark.parametrize("extension", ["asc", "csv"])
def test_replacement_does_not_accept_invalid_numeric_values(tmp_path, extension):
    record = Record.from_dict(
        {"content": [Question("NUMBER", "quantity", location=Location(1))]}
    )
    path = tmp_path / f"responses.{extension}"
    path.write_bytes(b"\x96")

    with pytest.raises(ValueError, match="invalid float literal"):
        record.validate_data(str(path))
    with pytest.raises(RuntimeError, match="ParseFloatError"):
        record.load_data(str(path))
