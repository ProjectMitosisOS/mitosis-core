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
