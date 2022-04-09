cmake_minimum_required(VERSION 3.2)
include( ExternalProject )

# depend

## ggflags
set(ggflags_DIR "${CMAKE_SOURCE_DIR}/../deps/gflags")
add_subdirectory(${CMAKE_SOURCE_DIR}/../deps/r2/deps/gflags gflags_dir)
