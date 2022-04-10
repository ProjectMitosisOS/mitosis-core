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
    for path in argv[1:] or os.listdir(BASE_DIR):
        if (
            not os.path.isdir(os.path.join(BASE_DIR, path)) or
            not os.path.exists(os.path.join(BASE_DIR, path, "tests"))
        ):
            continue

        print("+ [{}]".format(path))

        run(
            "make", "-C", BASE_DIR,
            "TEST_NAME={}".format(path.replace("-", "_")),
            "TEST_PATH={}".format(path),
        )


if __name__ == "__main__":
    main(sys.argv)
