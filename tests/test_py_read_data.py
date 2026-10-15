from triple_s import read_datamap
import polars as pl
import pytest
import warnings

PATH_SSS_XML = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"

@pytest.fixture
def datamap():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return read_datamap(PATH_SSS_XML)
    
def test_load_asc_file(datamap):
    datamap.load_data(PATH_ASC)
    df = datamap.to_polars()

    with pl.Config(
        tbl_rows=-1,          # Show ALL rows
        tbl_cols=-1,          # Show ALL columns
        fmt_str_lengths=20, # Show long string values
        tbl_width_chars=-1,   # No table width limit
    ):
        print(df)