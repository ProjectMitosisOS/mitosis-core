# DC/RC One-sided Read Benchmark

## Building

We first build the benchmarks and send binaries to remote hosts.

```bash
./compile.sh
ls *.ko
./sync_to_server.sh
```

## Benchmarks

### DC Throughput

We measure the DC one-sided read throughput with **12** threads per client machine and different payload size (64B and 4096B).

Change the `memory_size` in the `global_configs` in `dc-throughput.toml` to benchmark with different payload size.

#### Running

```bash
python3 ../../../scripts/bootstrap_proxy.py -f dc-throughput.toml -u USER -p PASSWORD
```

Remember to remove the kernel module after the benchmark.
```bash
python3 ../../../scripts/bootstrap_proxy.py -f dc-clean.toml -u USER -p PASSWORD
```

#### Reference Results

Sample Output:

```
...
@val00      [  +1.016451] src/lib.rs@156: [INFO ] - check global reporter states: 2788405, passed: 1016458. thpt : 2743256
@val00      [  +1.023991] src/lib.rs@156: [INFO ] - check global reporter states: 2817749, passed: 1023998. thpt : 2751713
@val00      [  +1.023994] src/lib.rs@156: [INFO ] - check global reporter states: 2812865, passed: 1023998. thpt : 2746943
@val00      [  +1.023994] src/lib.rs@156: [INFO ] - check global reporter states: 2815108, passed: 1023999. thpt : 2749131
...
```

The benchmark is conducted on val01 cluster. (Linux 4.15.0-46-generic #49~16.04.1-Ubuntu.)

64B payload

| #client machine | total throughput (op/s) | bandwidth (Gb/s) |
| --------------- | ----------------------- | ---------------- |
| 1               | 4074876 = 4.07M         | 1.94             |
| 4               | 9940997 = 9.94M         | 4.74             |
| 8               | 9944456 = 9.94M         | 4.74             |

4096B payload

| #client machine | total throughput (op/s) | bandwidth (Gb/s) |
| --------------- | ----------------------- | ---------------- |
| 1               | 2747098 = 2.74M         | 83.8             |
| 8               | 2941154 = 2.94M         | 89.7             |
