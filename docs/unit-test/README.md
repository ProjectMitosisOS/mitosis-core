# Mitosis Unit Tests

## Overview

Each module crate is equipped with several unit tests, including `mitosis/unitests`, `os-network/unitests`, and `mitosis-macros/unitests`. For example, we can run the unit tests under `mitosis/unitests` with the commands below.

```bash
cd mitosis/unitests
ls # show all the unit tests
# dc_pool fork prefetch ...
python run_tests.py # run all the unit tests
python run_tests.py fork # run one single unit test, do not include the '/' after the directory name
```

The successful unit tests will end with the following log lines.

```
running 1 test
test test_basic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s
```
