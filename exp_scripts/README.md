## Experiments on MITOSIS

### Building project

- All of the building process including the `rootfs` configuration (on child hosts)

| Command                               | Feature                     | Note                                                         |
| ------------------------------------- | --------------------------- | ------------------------------------------------------------ |
| `make build-cpp`                      | -                           | Build All of cpp executable files<br />Generate into directory `${PROJECT_PATH}/exp` |
| `make build-mitosis-prefetch`         | cow prefetch                | Configuration for default mitosis.                           |
| `make build-mitosis-prefetch-profile` | cow prefetch resume-profile | Show detailed memory/runtime latency profile in `dmesg`<br />Especially the memory consumtions. |
| `make build-mitosis-cow`              | cow                         | Exclude prefetch strategy                                    |
| `make build-mitosis-cow-prefetch`     | cow resume-profile          | COW mode w/ detailed profile infomation                      |
| `make build-mitosis-cache`            | cow page-cache              | Caching strategy                                             |
| `make build-mitosis-eager-resume`     | cow prefetch eager-resume   | Eager resume strategy (instead of on-demand fetch)           |



