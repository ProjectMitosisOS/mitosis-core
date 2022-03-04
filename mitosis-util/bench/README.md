# Benchmarks

## CPU Binding

This benchmark measures the throughput of an nop operation on a single machine with the benchmark framework.

TODO

### Reference Result

The benchmark is conducted on val01. (Linux val01 4.15.0-46-generic #49~16.04.1-Ubuntu.)

For a simple empty operation, the performance before and after the core binding is nearly the same.

The throughput for the empty operation is about **2130M op/s** (**4269M op/s**) when we spawn **12** (**24**) benchmark thread.
