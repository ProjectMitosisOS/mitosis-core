# RC Connection Benchmark

## Building

We first build the benchmarks and send binaries to remote hosts.

```bash
./compile.sh
ls *.ko # rc_conn_client_tests.ko and rc_conn_server_tests.ko
./sync_to_server.sh
```

## Benchmarks

### Latency

We measure the RC Connection latency with single thread and single NIC.

#### Running

```bash
python3 ../../../scripts/bootstrap_proxy.py -f latency/run.toml -u USER -p PASSWORD
```

Remember to remove the kernel module after the benchmark.
```bash
python3 ../../../scripts/bootstrap_proxy.py -f clean.toml -u USER -p PASSWORD
```

#### Reference Results

Sample Output:

```
...
@val04      wtx/mitosis/deps/rust-kernel-rdma/KRdmaKit/src/lib.rs@80: [INFO ] - KRdmaKit driver initialization done.
@val04      [  +0.000050] /mnt/hdd/wtx/mitosis/mitosis-util/src/bench.rs@103: [DEBUG] - Bench thread 0 started
@val04      [  +1.012494] src/lib.rs@124: [INFO ] - check global reporter states: 207, passed: 1012508. thpt : 204 # The single thread throughput is 204 op/s
@val04      [  +1.023990] src/lib.rs@124: [INFO ] - check global reporter states: 206, passed: 1023999. thpt : 201
...
```

The benchmark is conducted on val01 and val04. (Linux 4.15.0-46-generic #49~16.04.1-Ubuntu.)

The single thread average throughput is **197 connection/s**. The average latency is **5.07ms**.


### Throughput with Threads

We measure the RC Connection throughput with single NIC, 1 server machine and 1 client machine, varying the thread count.

#### Running

We can change the `thread_count` in `throughput-thread/run.toml` to conduct benchmarks with different threads.

```bash
python3 ../../../scripts/bootstrap_proxy.py -f throughput-thread/run.toml -u USER -p PASSWORD
```

Remember to remove the kernel module after the benchmark.
```bash
python3 ../../../scripts/bootstrap_proxy.py -f clean.toml -u USER -p PASSWORD
```

#### Reference Results

Sample Output:

```
@val04      [  +1.011946] src/lib.rs@124: [INFO ] - check global reporter states: 212, passed: 1011959. thpt : 209
@val04      [  +1.024018] src/lib.rs@124: [INFO ] - check global reporter states: 213, passed: 1024027. thpt : 208
@val04      [  +1.023990] src/lib.rs@124: [INFO ] - check global reporter states: 203, passed: 1023999. thpt : 198
```

The benchmark is conducted on val01 and val04. (Linux 4.15.0-46-generic #49~16.04.1-Ubuntu.)
The throughput peak is achieved with **2** threads.
| thread | throughput (connection/s) |
| ------ | ------------------------- |
| 1      | 172.6                     |
| 2      | 375.9                     |
| 4      | 398                       |
| 8      | 391                       |
| 12     | 400                       |

### Throughput with Machines

We measure the RC Connection throughput with single NIC, 1 server machine and multiple client machines. Each machine will have **2** threads.

#### Running

```bash
python3 ../../../scripts/bootstrap_proxy.py -f throughput-machine/run.toml -u USER -p PASSWORD # Test with all 8 machines
python3 ../../../scripts/bootstrap_proxy.py -f throughput-machine/run.toml -b val00 val04 val05 val06 -u USER -p PASSWORD # Test with 4 machines, use -b to add some machines to black list
```

Remember to remove the kernel module after each benchmark.
```bash
python3 ../../../scripts/bootstrap_proxy.py -f clean.toml -u USER -p PASSWORD
```

#### Reference Results

The benchmark is conducted on val. (val01 as the server, val00/val04-val10 as the clients)

| client machine | throughput (connection/s) |
| -------------  | ------------------------- |
| 1              | 397.3                     |
| 2              | 408.6                     |
| 4              | 400.8                     |
| 8              | 391.2                     |

### Throughput with Dual NICs

We measure the RC Connection throughput with dual NICs, 1 server machine with dual NICs and 1 client machine with dual NICs, varying the thread count.

The benchmark threads will use the NIC interleavedly. E.g.:
```
thread 0 will use local NIC 0 to connect the remote NIC 0

thread 1 will use local NIC 1 to connect the remote NIC 1

thread 2 will use local NIC 0 to connect the remote NIC 0

thread 3 will use local NIC 1 to connect the remote NIC 1
```
#### Running

We can change the `thread_count` in `dual-nic/run.toml` to conduct benchmarks with different threads.

```bash
python3 ../../../scripts/bootstrap_proxy.py -f dual-nic/run.toml -u USER -p PASSWORD
```

#### Reference Results

| thread | throughput (connection/s) |
| ------ | ------------------------- |
| 1*     | 173                       |
| 2      | 352                       |
| 4      | 802                       |
| 8      | 841                       |
| 12     | 845                       |

*: When the thread number is 1, we are actually using 1 nic
