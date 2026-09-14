import triple_s
from triple_s import *

path = "tests/sss_classic.sss"
survey = triple_s.read_sss(path)

print(help(survey.to_string))