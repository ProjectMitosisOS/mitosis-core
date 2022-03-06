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

We measure the RC Connection latency with single thread and single card.

#### Running

```bash
python3 ../../../scripts/bootstrap_proxy.py -f latency/connection_latency.toml -u USER -p PASSWORD
```

Remember to remove the kernel module after the benchmark.
```bash
python3 ../../../scripts/bootstrap_proxy.py -f clean.toml -u USER -p PASSWORD
```

#### Reference Results

Output:

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
