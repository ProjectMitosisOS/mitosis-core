#!/usr/bin/env python

import os
import subprocess
import sys

BASE_DIR = os.path.realpath(os.path.dirname(__file__))

def run(*args, **kwargs):
    cwd = kwargs.pop("cwd", None)
    environ = kwargs.pop("environ", os.environ)
    assert not kwargs

    print("+ [RUNNING] {}".format(list(args)))
    subprocess.check_call(list(args), cwd=cwd, env=environ)


def main(argv):
    print("I am in", BASE_DIR)
    for path in argv[1:] or os.listdir(BASE_DIR):
        if (
            not os.path.isdir(os.path.join(BASE_DIR, path))
        ):
            continue

        print("+ [{}]".format(path))

        print(path)

        run(
            "make", "-C", BASE_DIR,
            "TEST_NAME={}_tests".format(path.replace("-", "_")),
            "TEST_PATH={}".format(path),
        )
        print("\n ========= to next build ========= \n")


if __name__ == "__main__":
    main(sys.argv)
