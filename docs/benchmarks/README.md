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

### Prepare Time Benchmark

### Execution Time Benchmark

### Startup Time Benchmark

### Peak Throughput Benchmark
