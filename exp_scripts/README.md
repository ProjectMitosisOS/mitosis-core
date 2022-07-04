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
make build-pause-runner && make micro-warm-start-with-pause # warm start (w/ pause/unpause) throughput microbenchmark of different apps
```

### Some remarks

1. In benchmark **warm start (w/ pause/unpause) throughput microbenchmark of different apps**, the peak throughput will be achieved with different parallel configurations.

|             | achieves peak throughput w/ #number parallel containers |
|-------------|---------------------------------------------------------|
| helloworld  | 4                                                       |
| compression | 30                                                      |
| json        | 24                                                      |
| pyaes       | 24                                                      |
| chameleon   | 24                                                      |
| image       | 24                                                      |
| pagerank    | 24                                                      |
| recognition | 24                                                      |

## Run micros at once (MITOSIS)

```shell
make build-mitosis micro
```

## Run micros at once (MITOSIS-cache)

```shell
make build-mitosis-cache micro
```
