# Benchmarks

## CPU Binding

This benchmark measures the throughput of an nop operation on a single machine with the benchmark framework.

### Building & running 

Building:
```
cd PATH_TO_MITOSIS/mitosis-util/bench
../../scripts/bootstrap_proxy.py -f cpu_binding/scripts/build.toml -u USER -p PWD
```

Running:
```
cd PATH_TO_MITOSIS/mitosis-util/bench
../../scripts/bootstrap_proxy.py -f cpu_binding/scripts/run.toml -u USER -p PWD
../../scripts/bootstrap_proxy.py -f cpu_binding/scripts/clean.toml -u USER -p PWD 
```

Be sure the clean the modules after the running!


### Reference Result

The benchmark is conducted on val01. (Linux val01 4.15.0-46-generic #49~16.04.1-Ubuntu.)

For a simple empty operation, the performance before and after the core binding is nearly the same.

The throughput for the empty operation is about **2130M op/s** (**4269M op/s**) when we spawn **12** (**24**) benchmark thread.
