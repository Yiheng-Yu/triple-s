from triple_s import read_datamap, read_triple_s
import polars as pl
import pytest
import warnings

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"

PATH_SSS_EXAMPLE_2 = "tests/sss-examples/sss-2.0/example2.sss"
PATH_CSV = "tests/sss-examples/sss-2.0/example2.csv"


@pytest.fixture
def datamap():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return read_datamap(PATH_SSS_EXAMPLE_1)

def test_read_reada_asc():
    survey = read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC)
    assert survey.title == "Historic House Exit Survey<br/>First Wave"

def test_read_data_csv():
    survey = read_triple_s(PATH_SSS_EXAMPLE_2, PATH_CSV)
    assert survey.title == "Historic House Exit Survey<br/>First Wave"

def test_load_asc_file(datamap):
    datamap.load_data(PATH_ASC)
    df = datamap.record.data.to_polars()
    print()
    print(datamap.record.data)

    with pl.Config(
        tbl_rows=-1,          # Show ALL rows
        tbl_cols=-1,          # Show ALL columns
        fmt_str_lengths=20,   # Show long string values
        tbl_width_chars=-1,   # No table width limit
    ):
        print(df)