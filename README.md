# MITOSIS: A system primitive of fast remote fork

Mitosis is a kernel module that provides a new system primitive of fast remote fork based on RDMA.

## Overview

TODO: Describe the information for each rust crate and overall architecture here.

## Getting Started Instructions

### Prerequisite

- OS: Ubuntu16.04 (throughly tested, in general is irrelevant to the OS)
- Linux kernel: 4.15.0-46-generic (porting needed to fit other OSes)
- MLNX_OFED driver: 4.9-3.1.5.0 (throughly, use our modified driver in case to support DCT)
- Rustc: 1.60.0-nightly (71226d717 2022-02-04)
- Clang-9

Please refer to the document [here](./docs/setup.md) for how to configure these environments.

### Compile the mitosis

Assumptions: we have finished installing the dependencies described in the Prerequisite. 

```bash
make km ## building the kernel module
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

We have provided a simple demo on how to use the kernel module to remote fork a process.

1. Choose two machines, one as the parent machine and one as the child machine. Get the gid (RDMA address) of the parent machine.

```bash
show_gids
# DEV     PORT    INDEX   GID                                     IPv4            VER     DEV
# ---     ----    -----   ---                                     ------------    ---     ---
# mlx5_0  1       0       fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c                 v1
# mlx5_1  1       0       fe80:0000:0000:0000:ec0d:9a03:0078:6376                 v1
# n_gids_found=2
```

Mitosis uses the first RDMA nic by default, so we will use the gid `fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c` here.

2. Prepare the demo C++ programs on both machines.

```bash
cd samples/cpp
cmake .
make
ls parent client -hl
# -rwxrwxr-x child
# -rwxrwxr-x parent
```

```bash
cd exp
cmake .
make connector
```

3. Compile and insert the kernel module on both machines.

```bash
make km && make insmod
file /dev/mitosis-syscalls
# /dev/mitosis-syscalls: setuid, setgid, sticky, character special (238/0)
```

4. Run the connector on the child machine to let the child machine to connect to the parent machine.

```bash
cd exp
./connector -gid="fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c" -mac_id=0 -nic_id=0
```

5. Run the parent program on the parent machine.

```bash
cd samples/cpp
./parent # the parent program will print an increasing counter from 0 repeatedly
# the default identification for the parent program is 73
```

6. Run the client program on the client machine

```bash
cd samples/cpp
./child -mac_id=0 -handler_id=73 # the child will start printing the counter from 0 as if it has forked the parent program on machine 0 (val01) with id 73 from the point before it starts print the counter
```

7. Use Ctrl+C to kill the parent and child and use `make rmmod` to uninstall the kernel module.

## Testing and Benchmarking

TODO: separate to another file in ./docs

We have provided unit tests, stress tests, and benchmarks for mitosis.

### Unit Tests

We have provided several unit tests for each module crates. The documents for unit tests can be found [here](docs/unit-test/README.md).

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
