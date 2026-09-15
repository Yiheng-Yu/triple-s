import os
from pprint import pprint
import triple_s

print(os.path.dirname(__file__))
path = os.path.join(os.path.dirname(__file__), "sss_classic.sss")
sss = triple_s.read_sss(path)

# q = sss["AGE"]
# q.title = "what"
print(sss.record)