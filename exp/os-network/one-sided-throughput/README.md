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
| 1               | 4091195 = 4.09M         | 1.95             |
| 4               | 15763042 = 15.76M       | 7.51             |
| 8               | 31121141 = 31.12M       | 14.84            |

64B payload with **Outstanding Requests** (batch size = 12)

| #client machine | total throughput (op/s) | bandwidth (Gb/s) |
| --------------- | ----------------------- | ---------------- |
| 1               | 33415697 = 33.41M       | 15.93            |
| 4               | 104907457 = 104.90M     | 50.02            |
| 8               | 100043094 = 100.04M     | 47.70            |

4096B payload

| #client machine | total throughput (op/s) | bandwidth (Gb/s) |
| --------------- | ----------------------- | ---------------- |
| 1               | 2747098 = 2.74M         | 83.8             |
| 8               | 2941154 = 2.94M         | 89.7             |

### RC Throughput

We measure the RC one-sided read throughput similarly.

#### Running

```bash
python3 ../../../scripts/bootstrap_proxy.py -f rc-throughput.toml -u USER -p PASSWORD
```

Remember to remove the kernel module after the benchmark.
```bash
python3 ../../../scripts/bootstrap_proxy.py -f rc-clean.toml -u USER -p PASSWORD
```

#### Reference Results

Sample Output:

```
Omitted...
```

The benchmark is conducted on val01 cluster. (Linux 4.15.0-46-generic #49~16.04.1-Ubuntu.)

64B payload

| #client machine | total throughput (op/s) | bandwidth (Gb/s) |
| --------------- | ----------------------- | ---------------- |
| 1               | 4429191 = 4.43M         | 2.11             |
| 4               | 17247574 = 17.24M       | 8.22             |
| 8               | 33524966 = 33.52M       | 15.99            |

64B payload with **Outstanding Requests** (batch size = 12)

| #client machine | total throughput (op/s) | bandwidth (Gb/s) |
| --------------- | ----------------------- | ---------------- |
| 1               | 34032953 = 34.03M       | 16.22            |
| 4               | 110338498 = 110.33M     | 52.61            |
| 8               | 126198337 = 126.19M     | 60.17            |

4096B payload

| #client machine | total throughput (op/s) | bandwidth (Gb/s) |
| --------------- | ----------------------- | ---------------- |
| 1               | 2866641 = 2.86M         | 87.4             |
| 8               | 2947166 = 2.95M         | 89.9             |

## Conclusion

- With small payload size like 64B, the throughput of RC qp's one-sided read operation is **7%~8%** higher than that of DC qp.
- With small payload size like 64B and **Outstanding Requests** (batch size = 12) optimization, the peak throughput of RC qp's one-sided read operation is **26%** higher than that of DC qp (Measure with 8 clients and 1 server).
- With large payload size like 4096B, the gap is smaller. And the peak throughput is nearly the same.
