# Mitosis User-level Lean Container

## Usage

Currently, we can refer to the unit test in `tests/`.

```bash
mkdir build
cd build
cmake ..
make
sudo ./test_lean_container_template
sudo ./test_setup_lean_container
```

If we have prepared the rootfs in [README.md](../README.md), we can run a python application like this.

We need to specify the rootfs directory and the python script name in the command line.

And the python scripts are placed in `mitosis-user-libs/mitosis-lean-container/python-app-image`.

```bash
sudo ./test_start_app ../../.base/python-app/rootfs hello.py
```
