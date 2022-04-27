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

There are ample `toml` scripts and shell scripts under `scripts`, and you may find them helpful to ease the experiment process.

| path                          | usage                                                        | trigger point                                                |
| ----------------------------- | ------------------------------------------------------------ | ------------------------------------------------------------ |
| `build.toml`                  | Build and insmod all of the kernel modules                   | Call at setup only                                           |
| `build-exp.toml`              | Build all of the `cmake` related binaries                    | Call at setup only                                           |
| `prepare-lean-container.toml` | Do preparation steps required by setting up <br />lean-containers. Choose the child under<br /> `mitosis-user-libs/mitosis-lean-container/app/simple_child` as default. | Call at setup only. <br />Should `insmod` the kernel module first! |
| `clean.toml`                  | Cleanup the kernel module (rmmod) and the lean-containers    | Call at destroy only                                         |
| `run_lean_container.sh`       | The trigger script that start the specific command in lean-container. | Wrapped by `.toml` files                                     |
| scripts under `exp`           | All of the experiment `toml` scripts                         | Call at each time we <br />do the experiment                 |



### 3. Microbenchmarks

#### 3.1 Run in raw environment

##### 3.1.1 Setup



##### 3.1.2  Microbenchmark of `Prepare` stage





#### 3.2 Run within lean-containers

##### 3.2.1 Setup

We have to ensure several setups. 

1. All of the kernel modules (on each host) has been inserted.
2. At each child host, the lean-container environment should be ready.
   - Image build and export (to rootfs)

Above two steps require `build.toml` and `prepare-lean-container.toml`, and please ensure running them in order before you start.

i.e.

```sh
./bootstrap.py -f build.toml
./bootstrap.py -f prepare-lean-container.toml
```



##### 3.2.2 Microbenchmark of raw function



