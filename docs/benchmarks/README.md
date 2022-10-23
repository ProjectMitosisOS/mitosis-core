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

```bash
make micro-c-prepare
```

Output:

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

#### Function Benchmark (Python)

```bash
make micro-c-prepare
```

Output:

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

### Execution Time Benchmark

### Startup Time Benchmark

### Peak Throughput Benchmark
