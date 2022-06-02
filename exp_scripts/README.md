##### Build everything

```sh
make build-mitosis  # Build MITOSIS
make build-mitosis-cache  # Build MITOSIS-cache
```



## Run C microbenchmark

```sh
make micro-c-prepare # prepare microbench 
make micro-c-execution # prepare microbench
make micro-c-startup # startup lean-container

```

## Run micro-functions

```sh
make micro-function-prepare  # prepare microbench
make micro-function-execution  # execution microbench
make micro-function-startup # startup lean-container
```


## Run micros at once (MITOSIS)

```shell
make build-mitosis micro
```

## Run micros at once (MITOSIS-cache)

```shell
make build-mitosis-cache micro
```

