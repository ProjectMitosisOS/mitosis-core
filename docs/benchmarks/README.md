# Mitosis Benchmarks

## Overview

The benchmarks are run with one coordinator machine and several runner machines. You can run the benchmarks with one-click operation on the coordinator machine.

Before starting the benchmark, you need to fill in a makefile with custom information of the machines you are using.

```bash
# Assume we are in the root directory of mitosis repo
cd exp_scripts
cp makefile_template makefile # copy and modify your version of makefile later
```

Modify the key information below in the makefile.

```
### configurations ###

USER=
PWD=
PROJECT_PATH=projects/mos
PARENT_GID=fe80:0000:0000:0000:248a:0703:009c:7ca0
PARENT_HOST=val06
CHILD_HOSTS=val07
STR_CHILD_HOSTS='val07'

#USE_PROXY_COMMAND=false # true or false
USE_PROXY_COMMAND=true # true or false
```

| Parameter Name    | Meaning                                                                                                      | Example                                 |
|-------------------|--------------------------------------------------------------------------------------------------------------|-----------------------------------------|
| USER              | The username of your account, should be same on all machines involved                                        | username                                |
| PWD               | The password of your account, should be same on all machines involved                                        | password                                |
| PARENT_GID        | The gid of your RDMA-enable machine, can be queried by show_gids                                             | fe80:0000:0000:0000:248a:0703:009c:7ca0 |
| PARENT_HOST       | The hostname of the parent machine in a remote fork test                                                     | val01                                   |
| CHILD_HOSTS       | The hostnames of the child machines in a remote fork test                                                    | val02,val03                             |
| STR_CHILD_HOSTS   | The hostname string representation of the child machines, should be consistent with CHILD_HOSTS              | 'val02','val03'                         |
| USE_PROXY_COMMAND | If we should use the proxy command, set to true if the coordinator machine is outside the LAN of the cluster |                                         |

## Benchmarks

### `fork_prepare` Time Benchmark

This benchmark measures the `fork_prepare` latency of some typical functions/microbenchmark programs.

This benchmark requires 1 machine.

Sample configuration:

```
PROJECT_PATH=/mnt/hdd/wtx/mitosis
PARENT_GID=fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c
PARENT_HOST=val01
CHILD_HOSTS=
STR_CHILD_HOSTS=

FILTER=Prepare
```

#### Preparation Before the Benchmark

Build and insert the kernel module on the target machine.

```bash
make build-mitosis-prefetch
```

#### Run Micro Benchmark (C++)

The micro benchmark measures the `fork_prepare` latency of a C++ program which consumes a memory area which varies from 1MB ~ 1GB.

```bash
make micro-c-prepare
```

Output:

The column `trace` is the memory area size in byte. The `Prepare time` is the latency of the `fork_prepare` call.

```
     trace  data
----------  ---------------------------------------------
   1048576  (u'@val01     ', u'Prepare time = 0.607 ms')
            (u'@val01     ', u'Prepare time = 0.648 ms')
            (u'@val01     ', u'Prepare time = 0.604 ms')
            (u'@val01     ', u'Prepare time = 0.703 ms')
            (u'@val01     ', u'Prepare time = 0.753 ms')
            (u'@val01     ', u'Prepare time = 0.77 ms')
            (u'@val01     ', u'Prepare time = 0.652 ms')
            (u'@val01     ', u'Prepare time = 0.648 ms')
            (u'@val01     ', u'Prepare time = 0.641 ms')
   4194304  (u'@val01     ', u'Prepare time = 0.651 ms')
            (u'@val01     ', u'Prepare time = 0.794 ms')
            (u'@val01     ', u'Prepare time = 0.73 ms')
            (u'@val01     ', u'Prepare time = 0.727 ms')
            (u'@val01     ', u'Prepare time = 0.761 ms')
            (u'@val01     ', u'Prepare time = 0.779 ms')
            (u'@val01     ', u'Prepare time = 0.689 ms')
            (u'@val01     ', u'Prepare time = 0.755 ms')
            (u'@val01     ', u'Prepare time = 0.709 ms')
   8388608  (u'@val01     ', u'Prepare time = 0.9 ms')
            (u'@val01     ', u'Prepare time = 0.846 ms')
            (u'@val01     ', u'Prepare time = 0.819 ms')
            (u'@val01     ', u'Prepare time = 0.849 ms')
            (u'@val01     ', u'Prepare time = 0.904 ms')
            (u'@val01     ', u'Prepare time = 0.89 ms')
            (u'@val01     ', u'Prepare time = 0.808 ms')
            (u'@val01     ', u'Prepare time = 0.979 ms')
            (u'@val01     ', u'Prepare time = 0.921 ms')
  16777216  (u'@val01     ', u'Prepare time = 1.201 ms')
            (u'@val01     ', u'Prepare time = 1.169 ms')
            (u'@val01     ', u'Prepare time = 1.212 ms')
            (u'@val01     ', u'Prepare time = 1.182 ms')
            (u'@val01     ', u'Prepare time = 1.209 ms')
            (u'@val01     ', u'Prepare time = 1.224 ms')
            (u'@val01     ', u'Prepare time = 1.271 ms')
            (u'@val01     ', u'Prepare time = 1.235 ms')
            (u'@val01     ', u'Prepare time = 1.203 ms')
  33554432  (u'@val01     ', u'Prepare time = 1.932 ms')
            (u'@val01     ', u'Prepare time = 1.87 ms')
            (u'@val01     ', u'Prepare time = 2.076 ms')
            (u'@val01     ', u'Prepare time = 1.933 ms')
            (u'@val01     ', u'Prepare time = 1.917 ms')
            (u'@val01     ', u'Prepare time = 2.037 ms')
            (u'@val01     ', u'Prepare time = 2.023 ms')
            (u'@val01     ', u'Prepare time = 2.135 ms')
            (u'@val01     ', u'Prepare time = 1.917 ms')
  67108864  (u'@val01     ', u'Prepare time = 3.351 ms')
            (u'@val01     ', u'Prepare time = 3.438 ms')
            (u'@val01     ', u'Prepare time = 3.419 ms')
            (u'@val01     ', u'Prepare time = 3.334 ms')
            (u'@val01     ', u'Prepare time = 3.326 ms')
            (u'@val01     ', u'Prepare time = 3.362 ms')
            (u'@val01     ', u'Prepare time = 3.383 ms')
            (u'@val01     ', u'Prepare time = 3.451 ms')
            (u'@val01     ', u'Prepare time = 3.326 ms')
 134217728  (u'@val01     ', u'Prepare time = 6.427 ms')
            (u'@val01     ', u'Prepare time = 6.227 ms')
            (u'@val01     ', u'Prepare time = 6.324 ms')
            (u'@val01     ', u'Prepare time = 6.133 ms')
            (u'@val01     ', u'Prepare time = 6.173 ms')
            (u'@val01     ', u'Prepare time = 6.273 ms')
            (u'@val01     ', u'Prepare time = 6.108 ms')
            (u'@val01     ', u'Prepare time = 6.239 ms')
            (u'@val01     ', u'Prepare time = 6.213 ms')
 268435456  (u'@val01     ', u'Prepare time = 11.549 ms')
            (u'@val01     ', u'Prepare time = 12.586 ms')
            (u'@val01     ', u'Prepare time = 12.148 ms')
            (u'@val01     ', u'Prepare time = 11.789 ms')
            (u'@val01     ', u'Prepare time = 11.645 ms')
            (u'@val01     ', u'Prepare time = 11.705 ms')
            (u'@val01     ', u'Prepare time = 11.853 ms')
            (u'@val01     ', u'Prepare time = 11.78 ms')
            (u'@val01     ', u'Prepare time = 11.886 ms')
 536870912  (u'@val01     ', u'Prepare time = 21.653 ms')
            (u'@val01     ', u'Prepare time = 22.626 ms')
            (u'@val01     ', u'Prepare time = 21.439 ms')
            (u'@val01     ', u'Prepare time = 21.8 ms')
            (u'@val01     ', u'Prepare time = 21.462 ms')
            (u'@val01     ', u'Prepare time = 22.022 ms')
            (u'@val01     ', u'Prepare time = 22.662 ms')
            (u'@val01     ', u'Prepare time = 21.621 ms')
            (u'@val01     ', u'Prepare time = 21.705 ms')
1073741824  (u'@val01     ', u'Prepare time = 42.977 ms')
            (u'@val01     ', u'Prepare time = 42.723 ms')
            (u'@val01     ', u'Prepare time = 42.7 ms')
            (u'@val01     ', u'Prepare time = 41.979 ms')
            (u'@val01     ', u'Prepare time = 48.522 ms')
            (u'@val01     ', u'Prepare time = 41.043 ms')
            (u'@val01     ', u'Prepare time = 41.304 ms')
            (u'@val01     ', u'Prepare time = 40.746 ms')
            (u'@val01     ', u'Prepare time = 42.841 ms')
```

#### Run Function Benchmark (Python)

The function benchmark measures the `fork_prepare` latency of a Python program which executes a custom function.

```bash
make micro-c-prepare
```

Output:

The column `trace` is the function name, and the column `data` is the latency of the `fork_prepare` call.

```
trace        data
-----------  -------------------------------------------------------------
chameleon    (u'@val01     ', u'[chameleon-prepare] duration: 1.91 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.68 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.83 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.71 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.81 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.76 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.82 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.66 ms')
             (u'@val01     ', u'[chameleon-prepare] duration: 1.79 ms')
compression  (u'@val01     ', u'[compression-prepare] duration: 0.93 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.77 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.83 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.80 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.90 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.95 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.85 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.84 ms')
             (u'@val01     ', u'[compression-prepare] duration: 0.83 ms')
helloworld   (u'@val01     ', u'[helloworld-prepare] duration: 0.87 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.77 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.71 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.80 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.83 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.83 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.76 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.82 ms')
             (u'@val01     ', u'[helloworld-prepare] duration: 0.78 ms')
image        (u'@val01     ', u'[image-prepare] duration: 2.76 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.62 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.76 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.77 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.73 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.67 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.68 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.56 ms')
             (u'@val01     ', u'[image-prepare] duration: 2.81 ms')
json         (u'@val01     ', u'[json-prepare] duration: 0.85 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.76 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.75 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.79 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.83 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.81 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.89 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.80 ms')
             (u'@val01     ', u'[json-prepare] duration: 0.82 ms')
micro        (u'@val01     ', u'[micro-prepare] duration: 1.20 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.10 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.23 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.18 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.29 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.24 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.25 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.12 ms')
             (u'@val01     ', u'[micro-prepare] duration: 1.17 ms')
pagerank     (u'@val01     ', u'[pagerank-prepare] duration: 4.99 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.21 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.09 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.21 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.42 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.25 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.16 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.37 ms')
             (u'@val01     ', u'[pagerank-prepare] duration: 5.28 ms')
pyaes        (u'@val01     ', u'[pyaes-prepare] duration: 1.25 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 0.95 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 1.19 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 0.94 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 0.93 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 1.08 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 1.16 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 0.87 ms')
             (u'@val01     ', u'[pyaes-prepare] duration: 0.96 ms')
recognition  (u'@val01     ', u'[recognition-prepare] duration: 21.55 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 17.79 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 18.09 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 19.69 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 12.70 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 18.39 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 12.77 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 18.51 ms')
             (u'@val01     ', u'[recognition-prepare] duration: 23.16 ms')
```

#### After the benchmark

```bash
make clean
```

### `fork_resume` Time Benchmark

This benchmark measures the latency of `fork_resume` of some typical functions/microbenchmark programs.

This benchmark requires 2 machines.

Sample configuration:

```
PROJECT_PATH=/mnt/hdd/wtx/mitosis
PARENT_GID=fe80:0000:0000:0000:ec0d:9a03:00ca:2f4c
PARENT_HOST=val01
CHILD_HOSTS=val02
STR_CHILD_HOSTS='val02'

FILTER=latency
```

#### Preparation Before the Benchmark

Build and insert the kernel module on the target machine.

```bash
make build-mitosis-prefetch
```

#### Run Micro Benchmark (C++)

The micro benchmark measures the `fork_resume` latency of a C++ program which consumes a memory area which varies from 1MB ~ 1GB.

```bash
make micro-c-startup
```

Output:

The column `trace` is the memory area size in byte. The `latency` is the latency of the `fork_resume` call.

```
     trace  data
----------  ---------------------------------------------------------------------------------------------
   1048576  (u'@val02     ', u'[fh884d2ZEw8] Throughput: 341.280315 containers/sec, latency 2.930143 ms')
            (u'@val02     ', u'[fh884d2ZEw8] Throughput: 344.154927 containers/sec, latency 2.905668 ms')
            (u'@val02     ', u'[fh884d2ZEw8] Throughput: 347.096808 containers/sec, latency 2.881041 ms')
            (u'@val02     ', u'[fh884d2ZEw8] Throughput: 334.158129 containers/sec, latency 2.992595 ms')
   4194304  (u'@val02     ', u'[5XAMlsuVgnp] Throughput: 340.930971 containers/sec, latency 2.933145 ms')
            (u'@val02     ', u'[5XAMlsuVgnp] Throughput: 342.400114 containers/sec, latency 2.920560 ms')
            (u'@val02     ', u'[5XAMlsuVgnp] Throughput: 339.038465 containers/sec, latency 2.949518 ms')
            (u'@val02     ', u'[5XAMlsuVgnp] Throughput: 337.732546 containers/sec, latency 2.960923 ms')
   8388608  (u'@val02     ', u'[u5hNuno582v] Throughput: 340.429225 containers/sec, latency 2.937468 ms')
            (u'@val02     ', u'[u5hNuno582v] Throughput: 340.728250 containers/sec, latency 2.934890 ms')
            (u'@val02     ', u'[u5hNuno582v] Throughput: 338.670175 containers/sec, latency 2.952725 ms')
            (u'@val02     ', u'[u5hNuno582v] Throughput: 334.838941 containers/sec, latency 2.986510 ms')
  16777216  (u'@val02     ', u'[7L8H56P2J2r] Throughput: 332.352214 containers/sec, latency 3.008856 ms')
            (u'@val02     ', u'[7L8H56P2J2r] Throughput: 334.171131 containers/sec, latency 2.992479 ms')
            (u'@val02     ', u'[7L8H56P2J2r] Throughput: 331.298836 containers/sec, latency 3.018423 ms')
            (u'@val02     ', u'[7L8H56P2J2r] Throughput: 328.859266 containers/sec, latency 3.040814 ms')
  33554432  (u'@val02     ', u'[Um9cY28l1rt] Throughput: 318.898928 containers/sec, latency 3.135790 ms')
            (u'@val02     ', u'[Um9cY28l1rt] Throughput: 320.269402 containers/sec, latency 3.122371 ms')
            (u'@val02     ', u'[Um9cY28l1rt] Throughput: 319.712038 containers/sec, latency 3.127815 ms')
            (u'@val02     ', u'[Um9cY28l1rt] Throughput: 320.582276 containers/sec, latency 3.119324 ms')
  67108864  (u'@val02     ', u'[3Pf9D6Q97WS] Throughput: 302.234070 containers/sec, latency 3.308694 ms')
            (u'@val02     ', u'[3Pf9D6Q97WS] Throughput: 296.988601 containers/sec, latency 3.367133 ms')
            (u'@val02     ', u'[3Pf9D6Q97WS] Throughput: 300.900570 containers/sec, latency 3.323357 ms')
            (u'@val02     ', u'[3Pf9D6Q97WS] Throughput: 298.097264 containers/sec, latency 3.354610 ms')
 134217728  (u'@val02     ', u'[Vw3AJDw7yZS] Throughput: 265.479273 containers/sec, latency 3.766772 ms')
            (u'@val02     ', u'[Vw3AJDw7yZS] Throughput: 263.464618 containers/sec, latency 3.795576 ms')
            (u'@val02     ', u'[Vw3AJDw7yZS] Throughput: 266.038808 containers/sec, latency 3.758850 ms')
            (u'@val02     ', u'[Vw3AJDw7yZS] Throughput: 266.075933 containers/sec, latency 3.758326 ms')
 268435456  (u'@val02     ', u'[weli88qPp1R] Throughput: 216.318531 containers/sec, latency 4.622812 ms')
            (u'@val02     ', u'[weli88qPp1R] Throughput: 218.255370 containers/sec, latency 4.581789 ms')
            (u'@val02     ', u'[weli88qPp1R] Throughput: 218.326008 containers/sec, latency 4.580306 ms')
            (u'@val02     ', u'[weli88qPp1R] Throughput: 218.843217 containers/sec, latency 4.569481 ms')
 536870912  (u'@val02     ', u'[QJ7vgPnW36a] Throughput: 160.459570 containers/sec, latency 6.232099 ms')
            (u'@val02     ', u'[QJ7vgPnW36a] Throughput: 160.589908 containers/sec, latency 6.227041 ms')
            (u'@val02     ', u'[QJ7vgPnW36a] Throughput: 160.885611 containers/sec, latency 6.215596 ms')
            (u'@val02     ', u'[QJ7vgPnW36a] Throughput: 160.344900 containers/sec, latency 6.236556 ms')
1073741824  (u'@val02     ', u'[c8ZDg665G10] Throughput: 103.809670 containers/sec, latency 9.633014 ms')
            (u'@val02     ', u'[c8ZDg665G10] Throughput: 104.267500 containers/sec, latency 9.590716 ms')
            (u'@val02     ', u'[c8ZDg665G10] Throughput: 105.266238 containers/sec, latency 9.499722 ms')
            (u'@val02     ', u'[c8ZDg665G10] Throughput: 104.181445 containers/sec, latency 9.598638 ms')
```

#### Run Function Benchmark (Python)

The function benchmark measures the `fork_resume` latency of a Python program which executes a custom function.

```bash
make micro-function-prepare
```

Output:

The column `trace` is the function name, and the column `data` is the latency of the `fork_prepare` call.

```
trace        data
-----------  ---------------------------------------------------------------------------------------------
chameleon    (u'@val02     ', u'[Ex7q5pz1j6S] Throughput: 317.647351 containers/sec, latency 3.148145 ms')
             (u'@val02     ', u'[Ex7q5pz1j6S] Throughput: 320.392027 containers/sec, latency 3.121176 ms')
             (u'@val02     ', u'[Ex7q5pz1j6S] Throughput: 320.766281 containers/sec, latency 3.117535 ms')
             (u'@val02     ', u'[Ex7q5pz1j6S] Throughput: 319.364228 containers/sec, latency 3.131221 ms')
compression  (u'@val02     ', u'[wmUG0kcBbNz] Throughput: 338.265514 containers/sec, latency 2.956258 ms')
             (u'@val02     ', u'[wmUG0kcBbNz] Throughput: 337.389692 containers/sec, latency 2.963932 ms')
             (u'@val02     ', u'[wmUG0kcBbNz] Throughput: 333.905563 containers/sec, latency 2.994859 ms')
             (u'@val02     ', u'[wmUG0kcBbNz] Throughput: 334.926332 containers/sec, latency 2.985731 ms')
helloworld   (u'@val02     ', u'[80NRNe71d7m] Throughput: 344.031392 containers/sec, latency 2.906711 ms')
             (u'@val02     ', u'[80NRNe71d7m] Throughput: 342.058110 containers/sec, latency 2.923480 ms')
             (u'@val02     ', u'[80NRNe71d7m] Throughput: 337.062474 containers/sec, latency 2.966809 ms')
             (u'@val02     ', u'[80NRNe71d7m] Throughput: 339.960725 containers/sec, latency 2.941516 ms')
image        (u'@val02     ', u'[B0B9GQ23wm0] Throughput: 299.223308 containers/sec, latency 3.341986 ms')
             (u'@val02     ', u'[B0B9GQ23wm0] Throughput: 298.307521 containers/sec, latency 3.352245 ms')
             (u'@val02     ', u'[B0B9GQ23wm0] Throughput: 300.330972 containers/sec, latency 3.329660 ms')
             (u'@val02     ', u'[B0B9GQ23wm0] Throughput: 296.637589 containers/sec, latency 3.371117 ms')
json         (u'@val02     ', u'[hsDAqCOPd8u] Throughput: 341.083650 containers/sec, latency 2.931832 ms')
             (u'@val02     ', u'[hsDAqCOPd8u] Throughput: 340.812356 containers/sec, latency 2.934166 ms')
             (u'@val02     ', u'[hsDAqCOPd8u] Throughput: 340.447765 containers/sec, latency 2.937308 ms')
             (u'@val02     ', u'[hsDAqCOPd8u] Throughput: 334.993324 containers/sec, latency 2.985134 ms')
micro        (u'@val02     ', u'[GXC0Y17QSUw] Throughput: 334.043328 containers/sec, latency 2.993624 ms')
             (u'@val02     ', u'[GXC0Y17QSUw] Throughput: 331.892637 containers/sec, latency 3.013023 ms')
             (u'@val02     ', u'[GXC0Y17QSUw] Throughput: 334.278805 containers/sec, latency 2.991515 ms')
             (u'@val02     ', u'[GXC0Y17QSUw] Throughput: 333.642733 containers/sec, latency 2.997218 ms')
pagerank     (u'@val02     ', u'[25vhkS9IDXq] Throughput: 261.317639 containers/sec, latency 3.826760 ms')
             (u'@val02     ', u'[25vhkS9IDXq] Throughput: 262.682599 containers/sec, latency 3.806876 ms')
             (u'@val02     ', u'[25vhkS9IDXq] Throughput: 261.931913 containers/sec, latency 3.817786 ms')
             (u'@val02     ', u'[25vhkS9IDXq] Throughput: 262.427059 containers/sec, latency 3.810583 ms')
pyaes        (u'@val02     ', u'[Z2S7pOJFGK6] Throughput: 335.626058 containers/sec, latency 2.979506 ms')
             (u'@val02     ', u'[Z2S7pOJFGK6] Throughput: 334.392609 containers/sec, latency 2.990497 ms')
             (u'@val02     ', u'[Z2S7pOJFGK6] Throughput: 334.541551 containers/sec, latency 2.989165 ms')
             (u'@val02     ', u'[Z2S7pOJFGK6] Throughput: 331.589797 containers/sec, latency 3.015774 ms')
recognition  (u'@val02     ', u'[O6M1jn4uBDQ] Throughput: 153.589322 containers/sec, latency 6.510869 ms')
             (u'@val02     ', u'[O6M1jn4uBDQ] Throughput: 154.655940 containers/sec, latency 6.465966 ms')
             (u'@val02     ', u'[O6M1jn4uBDQ] Throughput: 155.336815 containers/sec, latency 6.437624 ms')
             (u'@val02     ', u'[O6M1jn4uBDQ] Throughput: 155.155182 containers/sec, latency 6.445160 ms')
```

### Execution Time Benchmark

This benchmark measures the execution time of the application after calling `fork_prepare`.

### Peak Throughput Benchmark
