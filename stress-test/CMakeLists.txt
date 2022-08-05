project(c_common)
ADD_DEFINITIONS(-std=c++17)
cmake_minimum_required(VERSION 3.2)

set(ggflags_DIR "${CMAKE_SOURCE_DIR}/../deps/gflags")
add_subdirectory(${CMAKE_SOURCE_DIR}/../deps/gflags gflags_dir)

include_directories(../mitosis-user-libs/mitosis-lean-container/lib)

set(targets
        simple_parent   ## simple parent demo
        )

add_executable(simple_parent fork/simple_parent.cc)
target_link_libraries(simple_parent gflags pthread -static)
