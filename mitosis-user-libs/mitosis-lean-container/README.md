# Lean Container

## Rootfs Preparation

Prepare your app according to the following structure. You **must** place your `Dockerfile` in the application directory.

A example is shown in `app/hello`.

```
└── hello
    ├── hello.py
    ├── requirements.txt
    └── Dockerfile # A Dockerfile is obligatory to build a docker image
```

We can build a docker image from this via `make_app_rootfs.py`. The name of created docker image specified by `--name`. The rootfs is exported to the path specified by `--export`.

```bash
export PATH_TO_APP=/path/to/app
export IMAGE_NAME=your_image_name
export ROOTFS_DIR=/path/to/rootfs
```

```bash
python3 make_app_rootfs.py --app $PATH_TO_APP --name $IMAGE_NAME --export $ROOTFS_DIR
```

For example, we can build a image named by `hello` from `app/hello`, and export rootfs to `.base/hello/rootfs` via the following command.

```bash
python3 make_app_rootfs.py --app app/hello/ --name hello --export .base/hello/rootfs
```

We can skip the build process of docker image, and only export the docker image.

```bash
python3 make_app_rootfs.py --name $IMAGE_NAME --only-export $ROOTFS_DIR
```

We can mount a device into the container's rootfs via `mount_device.py`.

```bash
export DEVICE=/dev/null
```

```bash
sudo python3 mount_device.py --rootfs $ROOTFS_DIR --device $DEVICE
```

And we can unmount it with the option `--unmount`.

```bash
sudo python3 mount_device.py --rootfs $ROOTFS_DIR --device $DEVICE --unmount
```

## Running the lean container

Build the lean container.

```bash
cd lib
mkdir build
cd build
cmake ..
make
```

Run the arbitrary binary in the lean container.

```bash
export CONTAINER_NAME=my_container
export ROOTFS_ABS_PATH=/path/to/rootfs
export COMMAND_ABS_PATH=/usr/bin/xxxx
export ARGS1=xxx
export ARGS2=xxx
```

```bash
sudo ./lib/build/start_lean_container $CONTAINER_NAME $ROOTFS_ABS_PATH $COMMAND_ABS_PATH $ARGS1 $ARGS2 # can be continued with arbitrary args
```

For example, we can run a python code `/hello.py` from the rootfs `${PWD}/.base/hello/rootfs/` via the following command.

Note that the rootfs should be specified with **absolute path** on the host machine, and the command should be specified with its **absolute path** in the rootfs directory.

```bash
sudo ./lib/build/start_lean_container my_test_container ${PWD}/.base/hello/rootfs/ /usr/local/bin/python /hello.py
```

## Performance of lean container
### Running the single thread microbenchmark

The single thread microbenchmark measures the latency of lean container creation.

The pseudo code of the critical path is shown below:

```plain
int critical_path() {
    int is_container = setup_container();
    if (is_container) {
        exit(0); # Exit immediately to avoid performance overhead
    } else {
        wait_container_exit();
    }
}
```

We will benchmark this critical path in the main process.

Build the lean container.

```bash
cd lib
mkdir build
cd build
cmake ..
make
```

Run the single thread benchmark of lean container.

```bash
sudo ./lib/build/benchmark_lean_container 10 # Running for 10 seconds
```

Reference output:

```plain
start 813 lean containers in 1.000936 second(s), latency per container 1.231164ms
start 818 lean containers in 1.000117 second(s), latency per container 1.222637ms
start 793 lean containers in 1.000598 second(s), latency per container 1.261789ms
start 828 lean containers in 1.000163 second(s), latency per container 1.207927ms
start 835 lean containers in 1.000285 second(s), latency per container 1.197946ms
start 806 lean containers in 1.000741 second(s), latency per container 1.241614ms
start 820 lean containers in 1.003585 second(s), latency per container 1.223884ms
start 826 lean containers in 1.001150 second(s), latency per container 1.212046ms
start 815 lean containers in 1.000556 second(s), latency per container 1.227677ms
total: start 8396 lean containers in 10.000178 second(s)
```

Reference performace: 1.2ms latency on val01.

### Running the concurrent container startup microbenchmark

We want to measure the concurrent container startup throughput of the lean container.

We benchmark the critical path above in multiple process and record the total throughput.

After building the lean container,

```bash
cd lib/scripts
./sync_to_server.sh
python3 ../../../../scripts/bootstrap_proxy.py -f lean_container_scalability.toml -u <username> -p <password>
```

Reference output:

```plain
@val01      start 832 lean containers in 1.000416 second(s), latency per container 1.202423ms
@val01      start 820 lean containers in 1.000961 second(s), latency per container 1.220684ms
@val01      start 841 lean containers in 1.000143 second(s), latency per container 1.189230ms
@val01      start 808 lean containers in 1.000902 second(s), latency per container 1.238740ms
@val01      start 809 lean containers in 1.000960 second(s), latency per container 1.237281ms
@val01      start 816 lean containers in 1.000040 second(s), latency per container 1.225539ms
```

We need to multiple the average throughput with the number of concurrent processes to get the final total output.

Reference performance:

The peak throughput on val01 is 5665 containers/s.

| number of concurrent processes | throughput |
| ------------------------------ | ---------- |
| 1                              | 832        |
| 6                              | 3864       |
| 10                             | 5382       |
| 12                             | 5665       |
| 16                             | 5491       |
| 24                             | 4625       |

## Running the lean container with CRIU

### Base Image Preparation

Build and export the criu base image to the specified path.

```bash
export ROOTFS_ABS_PATH=/path/to/rootfs
```

```bash
sudo python3 make_app_rootfs.py --app ./app/criu-base/ --name criu --export $ROOTFS_ABS_PATH
```

### Dump images on the host machine

```bash
cd app/criu
sudo bash host_dump.sh # TODO: this command will only work on val01
```

### Copy images and environments to the rootfs

```bash
cd app/criu
sudo bash copy_env.sh $ROOTFS_ABS_PATH # TODO: the val01-specific environments
```

### Restore process in the container

```bash
export CONTAINER_NAME=my_container
```

```bash
sudo ./lib/build/start_lean_container $CONTAINER_NAME $ROOTFS_ABS_PATH /bin/bash /restore.sh
```

Sample output:

```
this is the lean container launcher process!
this is the process in the lean container, pid in container: 2
restore!
xxxxx (Output of containered process)
```
