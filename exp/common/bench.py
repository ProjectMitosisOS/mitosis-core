import time
import sys

def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] duration: %.2f ms" % (str(name), passed_us / 1000))
    sys.stdout.flush()
