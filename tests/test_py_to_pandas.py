from datetime import date, time
import warnings

import pandas as pd
import pytest

from triple_s import read_datamap


@pytest.fixture
def survey():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        survey = read_datamap("tests/sss-examples/sss-2.0/example1.sss")
    return survey


def test_to_pandas_without_loaded_data(survey):
    with pytest.raises(RuntimeError, match="Data not loaded"):
        survey.to_pandas()


def test_loaded_data_to_pandas(survey):
    survey.load_data("tests/sss-examples/sss-2.0/example1.asc")
    result = survey.to_pandas()
    print(result)
    
    assert isinstance(result, pd.DataFrame)
    assert result.shape == (3, 12)
    assert result.columns.tolist() == [
        "RESPONDENT_ID", "Q1.a", "Q1.b", "Q2", "Q3", "Q3.a",
        "Q4", "Q5", "Q6", "Q7", "Q8", "WT",
    ]
    assert result["RESPONDENT_ID"].tolist() == [520001.0, 520002.0, 520003.0]
    assert result["Q1.a"].iloc[0] == date(2005, 5, 4)
    assert result["Q1.b"].iloc[0] == time(11, 20)
    assert result["Q2"].tolist() == [0, 2, 1]
    assert result["Q3"].iloc[0] == [1, 3, 5, 9]
    assert result["Q5"].iloc[0] == [1, 5]
    assert result["Q7"].tolist() == [True, False, True]
    assert result["Q8"].iloc[0] == "A"
    assert pd.isna(result["Q8"].iloc[1])
    assert result["Q8"].iloc[2] == "C"
    assert result["WT"].iloc[0] == pytest.approx(1.1310)

    pd.testing.assert_frame_equal(result, survey.record.to_pandas())
