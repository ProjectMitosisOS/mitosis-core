# Lean Container

## Rootfs Preparation

Suppose you have a directory named `app` and the application python code has put in it with the following structure: 

```
└── hello
    ├── hello.py
    ├── requirements.txt
    └── Dockerfile # Optional, we have a default Dockerfile
```

We can build a docker image from `hello/`. The name of docker image is `hello`. The rootfs is exported to `.base/hello/rootfs`.

```bash
python3 make_app_rootfs.py --app $PATH_TO_APP$/app/hello/ --name hello --export $OUTPUT_DIR$/$NAME$
```


We can skip the build process of docker image, and only export the docker image.

```bash
python3 make_app_rootfs.py --name hello --only-export $OUTPUT_DIR$/$NAME$
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

Run the python code in the lean container.

```bash
sudo ./lib/build/test_start_app $OUTPUT_DIR$/$NAME$ hello.py
```

## Running the single thread microbenchmark

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

## Running the concurrent container startup microbenchmark

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
