project(c_common)
ADD_DEFINITIONS(-std=c++17)
cmake_minimum_required(VERSION 3.2)

set(ggflags_DIR "${CMAKE_SOURCE_DIR}/../deps/gflags")
add_subdirectory(${CMAKE_SOURCE_DIR}/../deps/gflags gflags_dir)

include_directories(../include)
include_directories(../mitosis-user-libs/mitosis-lean-container/lib)

set(samples
        connector       ## Init connector
        simple_child    ## simple child demo
        simple_parent   ## simple parent demo
        bench_exe_time_parent ## get execution time
        bench_prepare_time  ## get prepare time
        bench_lean_container
        bench_process_fork
        bench_workingset ## C++ version of parent of the microbenchmark
        bench_workingset_exe ## C++ version of parent of the microbenchmark
        bench_nil_rpc
#        bench_file_access ## benchmark code to verify why CRIU-local is fast
        )

add_executable(connector common/connector.cc)
add_executable(simple_child common/simple_child.cc)
add_executable(simple_parent common/simple_parent.cc)
add_executable(bench_lean_container fork-functions/benchmark_lean_container.c
        ../mitosis-user-libs/mitosis-lean-container/lib/core/lean_container.c
        )
add_executable(bench_process_fork fork-functions/benchmark_process_fork.c)
add_executable(bench_exe_time_parent fork-micro/bench_exe_time_parent.cc)
add_executable(bench_prepare_time fork-micro/bench_prepare_time.cc)

add_executable(bench_workingset fork-functions/micro/workingset.cc fork-functions/micro/workingset_impl.cc)
add_executable(bench_workingset_exe fork-functions/micro/working_set_bench.cc fork-functions/micro/workingset_impl.cc)
#add_executable(bench_file_access fork-functions/micro/bench_file_access.cc fork-functions/micro/workingset_impl.cc)

add_executable(bench_nil_rpc os-network/rpc/bench_nil_rpc.cc)
add_library(micro_function SHARED fork-functions/micro/workingset_impl.cc)
set_property(TARGET micro_function PROPERTY POSITION_INDEPENDENT_CODE ON)

foreach (s ${samples})
    target_link_libraries(${s} gflags pthread)
endforeach (s)


