# MITOSIS: A system primitive of fast remote fork

Mitosis is a kernel module that provides a new system primitive of fast remote fork based on RDMA.

## Overview

TODO: Describe the information for each rust crate here.

## Getting Started Instructions

### Prerequisite

- OS: Ubuntu16.04
- Linux kernel: 4.15.0-46-generic
- MLNX_OFED driver: 4.9-3.1.5.0
- Rustc: 1.60.0-nightly (71226d717 2022-02-04)
- Clang-9

### Compile the project

```bash
make km
file mitosis-kms/fork.ko
# mitosis-kms/fork.ko: ELF 64-bit LSB relocatable, x86-64, version 1 (SYSV), BuildID[sha1]=xxx, not stripped
```

Mitosis has different configurations, including:
    - Prefetch: Read ahead some pages with RDMA
    - Page cache: Cache some pages locally instead of read through RDMA
    - COW: Use Copy-On-Write instead of directly copying page content
    - Eager resume: Read all the pages during the startup
    - Profile: Print performance profile during the execution
These configurations are specified in the `mitosis-kms/Kbuild` file with Rust features. Without further explanation, we will use the default configuration "COW+Prefetch". If you want to use other configurations, you can copy the Kbuild file before the compilation.

```bash
ls mitosis-kms/Kbuild* # will show the available Kbuild configurations
cp mitosis-kms/Kbuild-mitosis-prefetch mitosis-kms/Kbuild
```

### Example 

We have provided a simple demo to use the kernel module.

## Tests and Benchmarks

We have provided unit tests, stress tests, and benchmarks for mitosis.

### Unit Tests

Each module crate is equipped with several unit tests, including `mitosis/unitests`, `os-network/unitests`, and `mitosis-macros/unitests`. For example, we can run the unit tests under `mitosis/unitests` with the commands below.

```bash
cd mitosis/unitests
ls # show all the unit tests
# dc_pool fork prefetch ...
python run_tests.py # run all the unit tests
python run_tests.py fork #  run one single unit test, do not include the '/' after the directory name
```

The successful unit tests will end with the following log lines.

```
running 1 test
test test_basic ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.22s
```

### Stress Tests

We have provided stress tests for the following functionalities in mitosis.

- RPC
- Remote Fork

To run the stress tests, you should refer to the [README](stress-test/README.md) under stress-test directory.

### Benchmarks

The documents of benchmarks of mitosis can be found [here](docs/benchmarks/README.md).

## Contribution

Want to contribute to mitosis? We have some unfinished designs and implementations. Please refer to the document [here](docs/contribution/README.md).

## Related Projects

- [KRCORE](https://ipads.se.sjtu.edu.cn:1312/distributed-rdma-serverless/kernel-rdma/rust-kernel-rdma/-/tree/master/) is a kenrel-space RDMA library.

## License
This project is licensed under the XXX license.


## Credits 
- [nix](https://docs.rs/nix/latest/nix/)
- [tokio](https://tokio.rs)
