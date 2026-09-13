import triple_s

path = "tests/sss_classic.sss"
survey = triple_s.read_sss(path)

print(survey.content)