exe_path=../bench_lean_container

bench_sec=5
name=bench_lean_container
rootfs_path=/home/lfm/projects/mos/mitosis-user-libs/mitosis-lean-container/.base/hello/rootfs
command="/app/lean_child"

sudo $exe_path $bench_sec $name $rootfs_path $command 1 73
