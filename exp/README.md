Contains all of the experiments upon MITOSIS.

- `mitosis-kms`: Kernel modules for test (based on MITOSIS). Run `make km KMODULE_NAME=<directory_name>` to build the kernel module
  - `fork`: Fully encapsulation of MITOSIS
- `os-network`: Basic RDMA operation microbenchmark
    - `rc-connection`: TODO:xxxx
- `fork-micro`: All of the microbenchmark upon the fork critical path
    - `execution-time`: Exp of the influence of **different working-set memory** on the **execute time**
