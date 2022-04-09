project(c_common)
ADD_DEFINITIONS(-std=c++17)
cmake_minimum_required(VERSION 3.2)

set(ggflags_DIR "${CMAKE_SOURCE_DIR}/../../deps/gflags")
add_subdirectory(${CMAKE_SOURCE_DIR}/../../deps/gflags gflags_dir)

include_directories(../../include)
set(samples
        connector       ## Init connector
        simple_child    ## simple child demo
        simple_parent   ## simple parent demo
        bench_exe_time_parent ## get execution time
        bench_prepare_time  ## get prepare time
        )
add_executable(connector ../common/connector.cc)
add_executable(simple_child ../common/simple_child.cc)
add_executable(simple_parent ../common/simple_parent.cc)
add_executable(bench_exe_time_parent bench_exe_time_parent.cc)
add_executable(bench_prepare_time bench_prepare_time.cc)
foreach (s ${samples})
    target_link_libraries(${s} gflags)
endforeach (s)


