## Stress tests on MITOSIS

### Mitosis RPC

This test stresses on the RPC framework.
Multiple RPC clients repeatedly calls an empty RPC function to stress the RPC server.
This stress test can also be used as a benchmark over RPC framework.

General build and run commands:

```sh
make build-rpc-bench && make peak-nil-rpc-kernel-module
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

* `templates-build/template-build-rpc-bench.toml` for building the kernel module
* `templates-run/peak-nil-rpc-kernel-module.toml` for running the benchmark

You can adjust the parameters in `templates-run/peak-nil-rpc-kernel-module.toml`.

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

1. We use $RANDOM to generate a base of session id for each machine to avoid session id collision.
2. We use 1 server thread to receive and response to the rpc requests.
3. The rpc requests are sent synchronously.

A sample output is shown below.

```plain
...
(u'@val02     ', u'[  +0.000002] src/lib.rs@182: [INFO ] - starting rpc client')
(u'@val02     ', u'[  +0.000096] src/lib.rs@55: [INFO ] - start stress test client 0')
(u'@val02     ', u'[  +0.000002] src/lib.rs@56: [INFO ] - gid: fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c')
...
# peak throughput starts
(u'@val09     ', u'[  +1.023995] src/lib.rs@221: [INFO ] - passed: 1024001us, thpt: 131063 requests/s')
(u'@val05     ', u'[  +1.023970] src/lib.rs@221: [INFO ] - passed: 1023982us, thpt: 131024 requests/s')
(u'@val02     ', u'[  +1.023989] src/lib.rs@221: [INFO ] - passed: 1023995us, thpt: 130888 requests/s')
(u'@val12     ', u'[  +1.023987] src/lib.rs@221: [INFO ] - passed: 1023997us, thpt: 130727 requests/s')
(u'@val06     ', u'[  +1.023993] src/lib.rs@221: [INFO ] - passed: 1023999us, thpt: 130887 requests/s')
(u'@val03     ', u'[  +1.023990] src/lib.rs@221: [INFO ] - passed: 1024001us, thpt: 130877 requests/s')
(u'@val07     ', u'[  +1.024015] src/lib.rs@221: [INFO ] - passed: 1024025us, thpt: 130834 requests/s')
(u'@val14     ', u'[  +1.023983] src/lib.rs@221: [INFO ] - passed: 1023994us, thpt: 130741 requests/s')
(u'@val04     ', u'[  +1.023974] src/lib.rs@221: [INFO ] - passed: 1023983us, thpt: 131091 requests/s')
(u'@val08     ', u'[  +1.023988] src/lib.rs@221: [INFO ] - passed: 1023998us, thpt: 131082 requests/s')
(u'@val09     ', u'[  +1.023998] src/lib.rs@221: [INFO ] - passed: 1024004us, thpt: 130936 requests/s')
...
(u'@val02     ', u'[  +0.000003] krdma test framework dropped')
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
