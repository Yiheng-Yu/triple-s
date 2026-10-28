import csv
from datetime import date, time
from pathlib import Path
import warnings

import pandas as pd
import pytest

from triple_s import read_datamap, NoDataError


@pytest.fixture
def survey():
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        return read_datamap("tests/sss-examples/sss-2.0/example2.sss")


def test_load_csv_example(survey):
    survey.load_data("tests/sss-examples/sss-2.0/example2.csv")
    result = survey.to_pandas()
    assert result.shape == (3, 12)
    assert result["RESPONDENT_ID"].tolist() == [520001, 520002, 520003]
    assert result["Q1.a"].iloc[0] == date(2005, 5, 4)
    assert result["Q1.b"].iloc[0] == time(11, 20)
    assert result["Q2"].tolist() == [0, 2, 1]
    assert result["Q3"].tolist() == [[1, 3, 5, 9], [2], [1, 2, 9]]
    assert result["Q3.a"].tolist() == ["Nottingham Goose Fair", "", '"Heritage" Zone']
    assert result["Q4"].tolist() == [2, 9, 1]
    assert result["Q5"].tolist() == [[1, 5], [2], [2, 9]]
    assert result["Q6"].tolist() == [25, 100, 999]
    assert result["Q7"].tolist() == [True, False, True]
    assert result["Q8"].iloc[0] == "A"
    assert pd.isna(result["Q8"].iloc[1])
    assert result["Q8"].iloc[2] == "C"
    assert result["WT"].tolist() == pytest.approx([1.131, 0.9921, 1.0089])
    assert survey.to_polars().shape == (3, 12)


@pytest.mark.parametrize("skip", [0, 1, 2])
def test_csv_quoting_and_skip(tmp_path, survey, skip):
    with open("tests/sss-examples/sss-2.0/example2.csv", newline="") as source:
        rows = list(csv.reader(source))
    row = rows[1]
    row[6] = '  Cafe, "你好"\nsecond line  '
    row[11] = ""  # Empty numeric field is a missing response.
    path = tmp_path / "responses.csv"
    with path.open("w", newline="") as output:
        writer = csv.writer(output)
        for _ in range(skip):
            writer.writerow(["documentation"])
        writer.writerow(row)
    metadata = Path("tests/sss-examples/sss-2.0/example2.sss").read_text()
    datamap_path = tmp_path / "datamap.sss"
    datamap_path.write_text(metadata.replace('skip="1"', f'skip="{skip}"'))
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        survey = read_datamap(str(datamap_path))
    survey.record.load_data(str(path))
    result = survey.to_pandas()
    assert result.shape == (1, 12)
    assert result["Q3.a"].iloc[0] == row[6]
    assert pd.isna(result["WT"].iloc[0])


def test_csv_missing_field_reports_question(tmp_path, survey):
    path = tmp_path / "short.csv"
    path.write_text("header\n520001,20050504\n")
    with pytest.raises(ValueError, match="Q1.b.*field 3"):
        survey.load_data(str(path))
    with pytest.raises(NoDataError, match="Data not loaded"):
        survey.to_pandas()


def test_csv_invalid_value_reports_record_and_question(tmp_path, survey):
    data = Path("tests/sss-examples/sss-2.0/example2.csv").read_text()
    path = tmp_path / "invalid.csv"
    path.write_text(data.replace("20050504", "invalid"))
    with pytest.raises(ValueError):
        survey.load_data(str(path))


@pytest.mark.parametrize("skip", ["-1", "invalid"])
def test_invalid_skip_is_reported(tmp_path, skip):
    metadata = Path("tests/sss-examples/sss-2.0/example2.sss").read_text()
    path = tmp_path / "datamap.sss"
    path.write_text(metadata.replace('skip="1"', f'skip="{skip}"'))
    with warnings.catch_warnings():
        warnings.simplefilter("ignore")
        survey = read_datamap(str(path))
    with pytest.raises(ValueError, match="skip must be a non-negative integer"):
        survey.load_data("tests/sss-examples/sss-2.0/example2.csv")
