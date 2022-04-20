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
start 1016 lean containers in 1.000291 second(s), latency per container 0.984539ms
start 988 lean containers in 1.000358 second(s), latency per container 1.012508ms
start 999 lean containers in 1.000496 second(s), latency per container 1.001497ms
start 996 lean containers in 1.000042 second(s), latency per container 1.004058ms
start 1005 lean containers in 1.000414 second(s), latency per container 0.995437ms
start 993 lean containers in 1.000375 second(s), latency per container 1.007427ms
start 1004 lean containers in 1.000173 second(s), latency per container 0.996188ms
start 1002 lean containers in 1.000394 second(s), latency per container 0.998397ms
start 1015 lean containers in 1.001096 second(s), latency per container 0.986301ms
total: start 9996 lean containers in 10.000178 second(s)
```

Reference performace: 1ms latency on val01.
