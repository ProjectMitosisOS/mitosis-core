import sys
import time

import zerorpc


def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()


c = zerorpc.Client()
c.connect("tcp://127.0.0.1:8080")

start = time.time()
res = c.handle()
end = time.time()
report("trigger", start, end)
print("[trigger] result", res)