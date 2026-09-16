import os
from pprint import pprint
import triple_s

print(os.path.dirname(__file__))
path = os.path.join(os.path.dirname(__file__), "sss_classic.sss")
sss = triple_s.read_sss(path)

# print(dir(sss))

# q = sss["AGE"]
# q.title = "what"
# print(sss.record)

print(sss.meta.version)
sss.meta.version = "haha"
print(sss.meta.version)

# print(sss.meta.__dict__)