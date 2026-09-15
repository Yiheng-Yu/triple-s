import os
from pprint import pprint
import triple_s

path = os.path.join(os.path.dirname(__file__), "sss_classic.sss")
sss = triple_s.read_sss(path)

print(sss[3].response.pandas)
print(sss[0].response.pandas)