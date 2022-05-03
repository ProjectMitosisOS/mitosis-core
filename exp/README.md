## Experiments of MITOSIS

### 1. Brief introduction

Contains all of the experiments upon MITOSIS.

- `os-network`: Basic RDMA operation microbenchmark
    - `rc-connection`: Performance evalution on connection thpt/latency of RC primitive.
    
- `common`: Contains the common parent/child process. 

- `fork-function`: Microbenchmarks and function running performance (in lean-container) samples. 

- `fork-micro` (deprecated): All of the microbenchmark upon the fork critical path
  
    - `bench_prepare_time.cc` / `bench_prepare_time.py`: Exp of the influence of **different working-set memory** on the **prepare time** 
    - `bench_exe_time_parent.cc` / `bench_exe_time_parent.py`: Exp of the influence of **different working-set memory** on the **execution time**
    - Note: This directory has been deprecated. The new micro-benchmark has been moved into `fork-function/micro`.
    

Note: All of the child process could be boosted via the file `common/simple_child.cc`. And you can use the scripts under `scripts/` directly.



### 2. Common tools

There are sample `toml` scripts and shell scripts under `exp/scripts`, and you may find them helpful to ease the experiment process.

| path                          | usage                                                        | trigger point                                                |
| ----------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| `build.toml`                  | Build and insmod all of the kernel modules                   | Call at setup only                                           |
| `build-exp.toml`              | Build all of the `cmake` related binaries                    | Call at setup only                                           |
| `prepare-lean-container.toml` | Do preparation steps required by setting up <br />lean-containers. Choose the child under<br /> `mitosis-user-libs/mitosis-lean-container/app/simple_child` as default. | Call at setup only. <br />Should `insmod` the kernel module first! |
| `clean.toml`                  | Cleanup the kernel module (rmmod) and the lean-containers    | Call at destroy only                                         |
| `run_lean_container.sh`       | The trigger script that start the specific command in lean-container. | Wrapped by `.toml` files                                     |
| scripts under `exp`           | All of the experiment `toml` scripts                         | Call at each time we <br />do the experiment                 |

Moreover, prepare the `bootstrap.py` (or `bootstrap_proxy.py`) under `exp/scripts` (which can be found in in the $PATH_TO_MITOSIS/scripts$. And make sure you have synchronize all of the codes to the server (e.g., by preparing the code in NFS).



### 3. Microbenchmarks

#### 3.1 Run in raw environment

##### 3.1.1 Setup

Build `cpp` executable binary files.

```sh
./bootstrap.py -f build-exp.toml
```

Build kernel modules.

```
./bootstrap.py -f build.toml
```



##### 3.1.2  Microbenchmark

All of the testcases is triggered by `exp/scripts/micro_runner.py`. Before running the benchmarks, change the absolute path in file `micro_runner.py` .

The `trigger_bootstrap`  takes the directory path name on you own machine (all of the `r"^run.*?toml$"` are in this directory. Note that this path is `Your` machine's path, not the path on the server)

```python
if __name__ == '__main__':
    trigger_bootstrap('bootstrap.py',
                      [
													'your/own/dir/path1', 'your/own/dir/path2'
                      ]
                      )
```

For example, on my own laptop, the content should be:

```python
if __name__ == '__main__':
    trigger_bootstrap('bootstrap.py',
                      [
                          '/Users/lufangming/Documents/repos/mitosis/scripts/exp/fork-micro/prepare-execution',
                      ]
                      )
```

Since then, you are free to run 

```
python micro_runner.py
```

The output would be 

```
run-128M.toml
Execute on file run-128M.toml, prepare latency          14610.050000 us
Execute on file run-128M.toml, 0-th touch latency       167778.020000 us
Execute on file run-128M.toml, 1-th touch latency       79367.880000 us
Execute on file run-128M.toml, 2-th touch latency       222141.980000 us
=======================================================================================
run-8M.toml
Execute on file run-8M.toml, prepare latency            12562.990000 us
Execute on file run-8M.toml, 0-th touch latency         10573.150000 us
Execute on file run-8M.toml, 1-th touch latency         5105.970000 us
Execute on file run-8M.toml, 2-th touch latency         13680.930000 us
=======================================================================================
```





#### 3.2 Run within lean-containers

##### 3.2.1 Setup

We have to ensure several setups. 

1. All of the test `cpp` programs should be ready.
2. All of the kernel modules (on each host) have been inserted.
3. At each child host, the lean-container environment should be ready.
   - Image build and export (to rootfs)

Above two steps require `build.toml` and `prepare-lean-container.toml`, and please ensure running them in order before you start.

i.e.

```sh
./bootstrap.py -f build-exp.toml
./bootstrap.py -f build.toml
./bootstrap.py -f prepare-lean-container.toml
```



##### 3.2.2 Microbenchmark

At this case, all of the tests can be triggered by `exp/scripts/lean_runner.py`. You should also change the absolute path in `trigger_bootstrap` function.

Run the python program as below:

```sh
python lean_runner.py
```

Then the output would be:

```
run case: run-256M.toml
Execute on file run-256M.toml   qps 111.500000 op/s     latency 8.901090 ms
run case: run-1M.toml
Execute on file run-1M.toml     qps 295.000000 op/s     latency 3.387856 ms
run case: run-4M.toml
Execute on file run-4M.toml     qps 292.000000 op/s     latency 3.419538 ms
run case: run-1G.toml
Execute on file run-1G.toml     qps 49.250000 op/s      latency 23.206077 ms
run case: run-16M.toml
Execute on file run-16M.toml    qps 273.250000 op/s     latency 3.650224 ms
```

> Note: The `toml` scripts now is set to run on `val07`, `val08`. If you change the host, please change the `.toml` files as well.



### 4. MITOSIS framework for common python parents

In this section, we'll introduce the framework for MITOSIS benchamark. The file is under `exp/common/mitosis_wrapper.py` and you can find examples under `exp/fork-functions`.

The `mitosis_wrapper.py` defines two wrappers `@tick_execution_time` and `@mitosis_bench`. 

- The `@tick_execution_time` would tick the running time the wrappered function elapsed.
- The `@mitosis_bench` would run the inner-wrappered function three time:
  - First twice running shows the `cold-start` and `warm-start` execution time.
  - Then put one `prepare` process to store the MITOSIS-descriptor.
  - Run the third time, this shows the child execution time.

Please look in `mitosis_wrapper.py` to see more function `arg` settings.

Example of `helloworld` as below:

```python
import os
import sys
import time
import mmap

sys.path.append("../../common")  # include outer path
from mitosis_wrapper import *

os.environ['OPENBLAS_NUM_THREADS'] = '1'
os.environ['MKL_NUM_THREADS'] = '1'


@tick_execution_time
def handler():
    print("hello world")


@mitosis_bench
def bench():
    handler()


if __name__ == '__main__':
    bench()
```



### Cleanup

After all of the experiments have done, don't forget to rmmod the kernel modules and tmpfs configurations !

```sh
./bootstrap.py -f clean.toml
```

