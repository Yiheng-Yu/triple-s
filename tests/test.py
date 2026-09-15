import os
from pprint import pprint
import triple_s

path = os.path.join(os.path.dirname(__file__), "sss_classic.sss")
sss = triple_s.read_sss(path)

q = sss["FEATURES"]
q.title = "what"
print(q, type(q))