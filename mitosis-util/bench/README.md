# Benchmarks

## CPU Binding

This benchmark measures the throughput of an nop operation on a single machine with the benchmark framework.

```bash
python3 run_tests.py cpu_binding
ls -hl testmodule.ko # we should have the compiled kernel module under mitosis-util/bench
```

In `dmesg`

```
[  +1.032724] src/lib.rs@64: [INFO ] - check global reporter states: 1514367422, passed: 1032731. thpt : 1466000000 = 1466M
[  +1.023991] src/lib.rs@64: [INFO ] - check global reporter states: 1492003845, passed: 1023999. thpt : 1457000000 = 1457M
...
[  +1.023191] src/lib.rs@104: [INFO ] - check global reporter with cpu binding states: 1308955580, passed: 1023197. thpt : 1279000000 = 1279M
[  +1.023992] src/lib.rs@104: [INFO ] - check global reporter with cpu binding states: 1299641635, passed: 1023998. thpt : 1269000000 = 1269M
```

We can manually run the test with

```
sudo dmesg -c > /dev/null && sudo insmod testmodule.ko thread_count=12 && sudo rmmod testmodule.ko
# in another terminal
dmesg -wH
```

### Reference Result

The benchmark is conducted on val01. (Linux val01 4.15.0-46-generic #49~16.04.1-Ubuntu.)

For a simple empty operation, the performance before and after the core binding is nearly the same.

The throughput for the empty operation is about **1200M op/s ~ 2000M op/s** when we spawn **12** benchmark thread.

The throughput is unsteady because the cpu frequency drops after some repeated runs of benchmark.
