from triple_s import read_triple_s # NotSameSurvey
from triple_s.analysis import ToplineTable
import pytest

PATH_SSS_EXAMPLE_1 = "tests/sss-examples/sss-2.0/example1.sss"
PATH_ASC = "tests/sss-examples/sss-2.0/example1.asc"


@pytest.fixture
def survey():
    return read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC)


# @pytest.fixture
# def side(survey):
#     return survey["q2"]


# @pytest.fixture
# def top1(survey):
#     return survey["q3"]


# def test_create_toline_no_topbreak(side):
#     _ = ToplineTable(side)
#     _ = ToplineTable(side, None)
#     _ = ToplineTable(side, None)
#     _ = ToplineTable(side, ())
#     _ = ToplineTable(side, [])


# def test_create_topline_with_topbreak(side, top1):
#     _ = ToplineTable(side, [top1])
#     _ = ToplineTable(side, top1)
#     _ = ToplineTable(side, (s for s in [top1, top1]))


# def test_crete_topline_fail():
#     s1 = read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC)
#     s2 = read_triple_s(PATH_SSS_EXAMPLE_1, PATH_ASC)

#     side = s1["q2"]
#     top = s2["q3"]
#     with pytest.raises(NotSameSurvey):
#         ToplineTable(side, [top])
