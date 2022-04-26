import time


def report(name, start, end):
    passed_us = (end - start) * 1000000
    print("[%s] time: %.2f us" % (str(name), passed_us))


