import time

import zerorpc

c = zerorpc.Client()
c.connect("tcp://127.0.0.1:8080")
res = c.handle()
print(res)


