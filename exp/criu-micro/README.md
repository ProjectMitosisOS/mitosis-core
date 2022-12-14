# CRIU Microbenchmark

## Mount the file system

Choose one of the follow file system to mount.

Export your mount point in absolute path in variable `ROOTFS_ABS_PATH`.

```bash
mkdir ./.base # example path
export ROOTFS_ABS_PATH=${PWD}/.base # example path
```

### tmpfs

```
sudo mount -t tmpfs -o size=30G tmpfs ${ROOTFS_ABS_PATH}
```

### ceph

TODO

## Run the benchmark

Choose a name for the container and export an environment variable `CONTAINER_NAME`.

```bash
export CONTAINER_NAME=my_container
```

### Prepare the rootfs

Suppose you have built a criu base image according to [README.md](../../mitosis-user-libs/mitosis-lean-container/README.md) and you have a docker image called `criu`.

```bash
sudo python3 ../../mitosis-user-libs/mitosis-lean-container/make_app_rootfs.py --name criu --only-export ${ROOTFS_ABS_PATH}
```

### Dump on the host machine

Export the memory size (in byte) touched by benchmark in the environment variable `MEMORY_SIZE`.

```bash
export MEMORY_SIZE=16777216
```

```bash
sudo bash host_dump.sh ${MEMORY_SIZE}
# wait for 3 seconds...
```

Example output:

```plain
TARGET_PID=4441
...
(00.028233) Dumping finished successfully
```

The `00.028233` is the dump cost in second.

Copy the contents to the lean container rootfs.

```bash
sudo bash copy_env.sh ${ROOTFS_ABS_PATH}
```

### Restore in the lean container and benchmark the performance

```bash
sudo bash run_benchmark.sh ${CONTAINER_NAME} ${ROOTFS_ABS_PATH}
```

Example output:

```plain
before start lean container: 1652613157.910828956 # let this be point A
this is the process in the lean container, pid in container: 2
this is the lean container launcher process!
before criu restore: 1652613157.917168680 # let this be point B, then (B-A) is the lean container latency overhead
pass lean container unit test!
before start python handler: 1652613157.934134 # let this be point C, then (C-B) is the restore time
[micro-execution] time: 10312.56 us # this is the execution time
```

### Reference Performance

|                     | 1MB  | 4MB  | 16MB | 64MB | 256MB | 1024MB |
| ------------------- | ---- | ---- | ---- | ---- | ----- | ------ |
| dump (on tmpfs, ms) | 12.7 | 14.4 | 21.5 | 46.5 | 158.2 | 514.9  |
| lean container (ms) | 6    | 6    | 6    | 6    | 6     | 6      |
| restore (ms)        | 5    | 5    | 5.2  | 5.2  | 5.2   | 5.6    |
| execution (ms)      | 0.72 | 3    | 12.7 | 55   | 188   | 744    |

## Run the benchmark of rcopy

[rcopy](https://man7.org/linux/man-pages/man1/rcopy.1.html) is a tool to copy files with RDMA.

1. Enable IPOIB on the machines.
2. Follow the section `Dump on the host machine` to dump the process on the host machine (machine A). The images will be in directory `imgs/`.
3. On another machine (machine B) which will receive the images

```bash
mkdir imgs
rcopy # start rcopy server
```

4. Send the dumped images in the current machine A to machine B

```bash
export IMG_DIRECTORY=imgs
export MACHINE_IP=<rdma ip of machine B>
bash rcopy.sh $IMG_DIRECTORY $MACHINE_IP
```

### Reference Performance

We use `time bash rcopy.sh $IMG_DIRECTORY $MACHINE_IP` to measure the time.

|                               | 1MB  | 4MB  | 16MB | 64MB | 256MB | 1024MB |
| ----------------------------- | ---- | ---- | ---- | ---- | ----- | ------ |
| rcopy tmpfs (ms)              | 212  | 210  | 214  | 232  | 377   | 890    |
| scp tmps (for reference) (ms) | 300  | 326  | 433  | 860  | 2574  | 9434   |

## Combined Results for tmpfs

|                     | 1MB  | 4MB  | 16MB | 64MB | 256MB | 1024MB |
| ------------------- | ---- | ---- | ---- | ---- | ----- | ------ |
| dump (on tmpfs, ms) | 12.7 | 14.4 | 21.5 | 46.5 | 158.2 | 514.9  |
| rcopy (ms)          | 212  | 210  | 214  | 232  | 377   | 890    |
| lean container (ms) | 6    | 6    | 6    | 6    | 6     | 6      |
| restore (ms)        | 5    | 5    | 5.2  | 5.2  | 5.2   | 5.6    |
| execution (ms)      | 0.72 | 3    | 12.7 | 55   | 188   | 744    |
