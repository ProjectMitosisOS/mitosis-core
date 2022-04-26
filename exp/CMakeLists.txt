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
        lean_child
        bench_lean_container
        )
add_executable(connector common/connector.cc)
add_executable(simple_child common/simple_child.cc)
add_executable(lean_child fork-functions/child/lean_child.cc)
add_executable(simple_parent common/simple_parent.cc)
add_executable(bench_lean_container fork-functions/benchmark_lean_container.c
        ../mitosis-user-libs/mitosis-lean-container/lib/core/lean_container.c
        )
add_executable(bench_exe_time_parent fork-micro/bench_exe_time_parent.cc)
add_executable(bench_prepare_time fork-micro/bench_prepare_time.cc)

foreach (s ${samples})
    target_link_libraries(${s} gflags)
endforeach (s)


