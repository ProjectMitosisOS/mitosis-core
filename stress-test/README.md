## Stress tests on MITOSIS

[toc]

### Mitosis RPC

#### Empty RPC

This test stresses on the RPC framework.
Multiple RPC clients repeatedly calls an empty RPC function to stress the RPC server.
This stress test can also be used as a benchmark over RPC framework.

General build and run commands:

```sh
make build-rpc-bench && make peak-rpc-kernel-module
```

Note that the experiments require one server (e.g.: val01) and multiple clients (e.g.: val02,val03, etc.). Fill it in the `makefile`.
You should first start from the `makefile_template`.

```bash
cp makefile_template makefile
```

Then you should fill in these parameters in the `makefile`. The `USER` and `PWD` is the username and the password on all the machines. The `PROJECT_PATH` is the absolute path of mitosis project on each machine. The `PARENT_GID` is the gid of the server. A sample configuration is shown below.

```
PARENT_GID=fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c
PARENT_HOST=val01
CHILD_HOSTS=val02,val03,val04,val05,val06,val07,val08,val09,val12,val14
STR_CHILD_HOSTS='val02','val03','val04','val05','val06','val07','val08','val09','val12','val14'
```

Related tomls:

* `templates-build/template-build-rpc-bench.toml` for building the kernel module
* `templates-run/peak-rpc-kernel-module.toml` for running the benchmark

You can adjust the parameters in `templates-run/peak-rpc-kernel-module.toml`.

```plain
server_running_secs = 80 # the server will run for 80 seconds
client_running_secs = 60 # should be smaller than the server_running_secs to ensure graceful exits of clients
thread_count = 12 # the kernel thread running on a single machine, 12 is suitable to achieve peak throughput
test_rpc_id = 778
server_qd_hint = 73
server_service_id = 0
client_qd_hint = 75
client_service_id_base = 1
report_interval = 1 # report throughput every 1 second
```

Remark:

1. We use $RANDOM$ to generate a base of session id for each machine to avoid session id collision.
2. We use 1 server thread to receive and response to the rpc requests.
3. The rpc requests are sent synchronously.

A sample output is shown below.

```plain
...
@val02     [  +0.000002] src/lib.rs@182: [INFO ] - starting rpc client
@val02     [  +0.000096] src/lib.rs@55: [INFO ] - start stress test client 0
@val02     [  +0.000002] src/lib.rs@56: [INFO ] - gid: fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c
...
# peak throughput starts
@val09     [  +1.023995] src/lib.rs@221: [INFO ] - passed: 1024001us, thpt: 131063 requests/s
@val05     [  +1.023970] src/lib.rs@221: [INFO ] - passed: 1023982us, thpt: 131024 requests/s
@val02     [  +1.023989] src/lib.rs@221: [INFO ] - passed: 1023995us, thpt: 130888 requests/s
@val12     [  +1.023987] src/lib.rs@221: [INFO ] - passed: 1023997us, thpt: 130727 requests/s
@val06     [  +1.023993] src/lib.rs@221: [INFO ] - passed: 1023999us, thpt: 130887 requests/s
@val03     [  +1.023990] src/lib.rs@221: [INFO ] - passed: 1024001us, thpt: 130877 requests/s
@val07     [  +1.024015] src/lib.rs@221: [INFO ] - passed: 1024025us, thpt: 130834 requests/s
@val14     [  +1.023983] src/lib.rs@221: [INFO ] - passed: 1023994us, thpt: 130741 requests/s
@val04     [  +1.023974] src/lib.rs@221: [INFO ] - passed: 1023983us, thpt: 131091 requests/s
@val08     [  +1.023988] src/lib.rs@221: [INFO ] - passed: 1023998us, thpt: 131082 requests/s
@val09     [  +1.023998] src/lib.rs@221: [INFO ] - passed: 1024004us, thpt: 130936 requests/s
...
@val02     [  +0.000003] krdma test framework dropped
```

The log prints the total throughput of one machine. For example, the average throughput per machine is **130922** requests/s and
there are 10 clients currently running the benchmark. The whole throughput is **130922x10=1309220** requests/s.

Reference throughput on val01 (server) and val02-val09,val12,val14 (clients), each with 12 threads

| number of client machines | rpc throughput (requests/s) |
|---------------------------|-----------------------------|
| 1                         | 1259365                     |
| 2                         | 1309660                     |
| 4                         | 1221549                     |
| 8                         | 1219961                     |
| 10                        | 1261974                     |

Reference latency on val01 (server) and val02 (client) with 1 thread

| single thread rpc throughput (requests/s) | latency (μs) |
|-------------------------------------------|--------------|
| 239952.4                                  | 4.16         |

#### RPC with checksum

This test stresses on the RPC framework. The payload (default 2048 bytes plus a 8-byte checksum) is checksumed to ensure the integrity of the payload data. The stress test server will generate a random payload with checksum each time and the client will calculate the checksum to ensure the integrity.

General build and run commands:

```sh
make build-rpc-checksum && make peak-rpc-kernel-module
```

Note that the experiments require one server (e.g.: val01) and multiple clients (e.g.: val02,val03, etc.). Fill it in the `makefile`.

The `PARENT_GID` is the gid of the server.

```
PARENT_GID=fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c
PARENT_HOST=val01
CHILD_HOSTS=val02,val03,val04,val05,val06,val07,val08,val09,val12,val14
STR_CHILD_HOSTS='val02','val03','val04','val05','val06','val07','val08','val09','val12','val14'
```

Related tomls:

* `templates-build/template-build-rpc-checksum.toml` for building the kernel module with checksum support
* `templates-run/peak-rpc-kernel-module.toml` for running the benchmark

Remark:

1. Adjust the `DEFAULT_PAYLOAD_SIZE` in `stress-test/rpc/rpc_common/src/payload.rs` change the payload size.
2. It passes the stress test if the output does not contain `ERROR`.

A sample output is shown below.

```plain
...
@val07     [  +1.004007] src/lib.rs@245: [INFO ] - passed: 1033670us, thpt: 12086 requests/s
@val02     [  +1.023991] src/lib.rs@245: [INFO ] - passed: 1024000us, thpt: 12346 requests/s
@val08     [  +0.998663] src/lib.rs@245: [INFO ] - passed: 1024045us, thpt: 10817 requests/s
...
```

### Mitosis Remote Fork

#### Simple C++ program stress test

This test stresses the mitosis remote fork with a simple C++ program as the parent process.

```
cd stress-test
make build-lean-container-bench # prepare the stress test
make peak-lean-container # run the stress test
make clean # clean the environment
```

Related tomls:

* `templates-build/template-build-cpp.toml` for building the related C++ program
* `templates-build/template-build-mitosis.toml` for building the mitosis with default configuration (only COW)
* `templates-run/peak-lean-container.toml` for running the stress test
* `templates-build/template-clean.toml` for cleaning the environment

Remark:

1. The stress test should run without error with 1 server machine and 10 client machines.
2. The output throughput should be steady without sudden drop (drop to <0.1 containers/sec).
3. The `dmesg` on each machine should not contain error. (should be checked manually).
4. You can adjust the Kbuild in `templates-build/template-build-mitosis.toml` to run mitosis with other configurations.

A sample **correct** output is shown below.

```
@val07     [660K5BZDgBs] Throughput: 120.045924 containers/sec, latency 8.330145 ms
@val07     [660K5BZDgBs] Throughput: 121.862228 containers/sec, latency 8.205988 ms
@val07     [660K5BZDgBs] Throughput: 120.813350 containers/sec, latency 8.277231 ms
@val07     [660K5BZDgBs] Throughput: 120.046682 containers/sec, latency 8.330093 ms
```

A typical **error** output is shown below,
and the `dmesg` will print error like `[ERROR] failed to create ah` or `RPC handler meets an error Error(Fatal)`.

```
@val07     [660K5BZDgBs] Throughput: 0.090090 containers/sec, latency xxxxxx ms
@val07     [660K5BZDgBs] Throughput: 0.090090 containers/sec, latency xxxxxx ms
```

#### Python application stress test

This test stresses the mitosis remote fork with different python applications.

This test is under the directory `exp_scripts`.

```
cd exp_scripts
make build-mitosis-cow # build the mitosis
make peak-func-lean-container # run the stress test
make clean # clean the environment
```

Related tomls (under `exp_scripts`):

* `templates-run/many-to-one-func/template-lean-container.toml` for running the stress test
* other tomls for building and cleaning the test are emitted


Remark:

1. You can adjust the `run_sec` variable in `templates-run/many-to-one-func/template-lean-container.toml` to control the running time of the stress test. The `run_sec` is the stress test time in seconds for each function. The startup and cleanup time of the stress test is not included in this time, so it will take longer time to run the stress test of each function.
2. Comment out some function names in `micro_func_name` in `templates-run/many-to-one-func/template-lean-container.toml` if you do not want to run them. The full version of stress test will last for at least 15 minutes (for all the 9 functions and `run_sec` is 60 seconds).
3. Same remarks with the C++ program stress test.

A sample output is shown below.

```
@val14     [T6U0WuFz614] Throughput: 2.276335 containers/sec, latency 439.302601 ms
@val14     [z367u4QL3gh] Throughput: 2.583709 containers/sec, latency 387.040498 ms
@val14     [0VXd9kuG12t] Throughput: 2.243970 containers/sec, latency 445.638739 ms
@val14     [5DbE54y7v49] Throughput: 1.919630 containers/sec, latency 520.933842 ms
```
