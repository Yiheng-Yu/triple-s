import triple_s
from triple_s import *

path = "tests/sss_classic.sss"
survey = triple_s.read_sss(path)

question = survey.record.content[0]
original_text = survey.record.content[0].text.copy()
print(f"Question text before update: {original_text}")
question.text = "Updated!!"
print(f"Question text after update: {survey.record.content[0].text}")
