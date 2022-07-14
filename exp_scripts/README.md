## Experiments on MITOSIS

### Building project

- All of the building process including the `rootfs` configuration (on child hosts)

| Command                               | Feature                     | Note                                                         |
| ------------------------------------- | --------------------------- | ------------------------------------------------------------ |
| `make build-cpp`                      | -                           | Build All of cpp executable files<br />Generate into directory `${PROJECT_PATH}/exp` |
| `make build-mitosis-prefetch`         | cow prefetch                | Configuration for default mitosis.                           |
| `make build-mitosis-prefetch-profile` | cow prefetch resume-profile | Show detailed memory/runtime latency profile in `dmesg`<br />Especially the memory consumtions. |
| `make build-mitosis-cow`              | cow                         | Exclude prefetch strategy                                    |
| `make build-mitosis-cow-prefetch`     | cow resume-profile          | COW mode w/ detailed profile infomation                      |
| `make build-mitosis-cache`            | cow page-cache              | Caching strategy                                             |
| `make build-mitosis-eager-resume`     | cow prefetch eager-resume   | Eager resume strategy (instead of on-demand fetch)           |

### Microbenchmarks

#### Mitosis RPC Throughput

General build and run commands:

```sh
make build-rpc-bench && make peak-nil-rpc-kernel-module
```

Note that the experiments requires one server (e.g.: val01) and multiple clients (e.g.: val02,val03, etc.). Fill it in the `makefile`.

The `PARENT_GID` is the gid of the server.

```
PARENT_GID=fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c
PARENT_HOST=val01
CHILD_HOSTS=val02,val03,val04,val05,val06,val07,val08,val09,val12,val14
STR_CHILD_HOSTS='val02','val03','val04','val05','val06','val07','val08','val09','val12','val14'
```

Related tomls:

* `templates-build/template-build-rpc-bench.toml` for building the kernel module
* `templates-run/execution-peak/peak-nil-rpc-kernel-module.toml` for running the benchmark

You can adjust the parameters in `templates-run/execution-peak/peak-nil-rpc-kernel-module.toml`.

```plain
server_running_secs = 80
client_running_secs = 60 # should be smaller than the server_running_secs to ensure graceful exits of clients
thread_count = 12 # the kernel thread running on a single machine, 12 is suitable
test_rpc_id = 778
server_qd_hint = 73
server_service_id = 0
client_qd_hint = 75
client_service_id_base = 1
report_interval = 1 # report every 1 second
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

Reference throughput on val01 (server) and val02-val09,val12,val14 (clients).

|    | rpc throughput (requests/s) |
|----|-----------------------------|
| 1  | 1259365                     |
| 2  | 1309660                     |
| 4  | 1221549                     |
| 8  | 1219961                     |
| 10 | 1261974                     |

### Baselines

#### warm start (w/ pause/unpause) throughput of different apps

General build and run commands: 

```sh
make build-pause-runner && make micro-warm-start-with-pause
```

Note that the experiments only requires one machine. Thus, we use the `parent` to execute all of them.

Related tomls:
* `templates-build/template-build-pause-runner.toml` for building benchmark runner program
* `templates-run/micro-func/template-run-micro-warm-start-with-pause.toml` for running the benchmark

Remarks:

1. Different app will achieve peak throughput with different parallel configurations.

|             | achieves peak throughput w/ #number parallel containers |
|-------------|---------------------------------------------------------|
| helloworld  | 4                                                       |
| compression | 30                                                      |
| json        | 24                                                      |
| pyaes       | 24                                                      |
| chameleon   | 24                                                      |
| image       | 24                                                      |
| pagerank    | 24                                                      |
| recognition | 24                                                      |

Change the parallel_containers in `exp_scripts/templates-run/micro-func/template-run-micro-warm-start-with-pause.toml` and alter
the number of toml entries with `benchmark_lean_container_pause_w_command` to benchmark with different number of parallel containers.

2. The final output log is in `exp_scripts/out/micro-warm-start-with-pause/run-<app name>.toml.txt`, where we should pick out the peak throughput
manually.

A sample output log is like below.

```plain
...
(u'@val01     ', u'/tmp-mitosis/app22/rootfs//uds.socket')
(u'@val01     ', u'this is the lean container launcher process!')
(u'@val01     ', u'/tmp-mitosis/app23/rootfs//uds.socket')
# Now all the containers have been started
# peak throughput starts
(u'@val01     ', u'pause/unpause 2388 lean containers in 1.000026 second(s), latency per container 0.418771ms')
(u'@val01     ', u'pause/unpause 2470 lean containers in 1.000010 second(s), latency per container 0.404862ms')
(u'@val01     ', u'pause/unpause 2192 lean containers in 1.000297 second(s), latency per container 0.456340ms')
(u'@val01     ', u'pause/unpause 2393 lean containers in 1.000134 second(s), latency per container 0.417941ms')
(u'@val01     ', u'pause/unpause 2231 lean containers in 1.000252 second(s), latency per container 0.448342ms')
(u'@val01     ', u'pause/unpause 2423 lean containers in 1.000281 second(s), latency per container 0.412828ms')
(u'@val01     ', u'pause/unpause 2263 lean containers in 1.000104 second(s), latency per container 0.441937ms')
(u'@val01     ', u'pause/unpause 2339 lean containers in 1.000119 second(s), latency per container 0.427584ms')
(u'@val01     ', u'pause/unpause 2354 lean containers in 1.000323 second(s), latency per container 0.424946ms')
# ...
# peak throughput ends when some client exits
(u'@val01     ', u'pass lean container unit test!')
('exit ', u'val01')
```

We should calculate the average throughput manually. E.g.: the average per-container throughput is 2338/s, and
there are 24 parallel containers. The final throughput is **24x2338=56112/s**

3. The benchmark is expected to run for 17-20 minutes on val01.

4. Reference results on val01: (This benchmark is expected to run on a single machine)

|             | achieves peak throughput w/ #number parallel containers | total throughput |
|-------------|---------------------------------------------------------|------------------|
| helloworld  | 4                                                       | 80288            |
| compression | 30                                                      | 7556             |
| json        | 24                                                      | 1651             |
| pyaes       | 24                                                      | 156              |
| chameleon   | 24                                                      | 75               |
| image       | 24                                                      | 143              |
| pagerank    | 24                                                      | 24               |
| recognition | 24                                                      | 60               |
