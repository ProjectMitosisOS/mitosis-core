project(simple_c_examples)
ADD_DEFINITIONS(-std=c++17)
cmake_minimum_required(VERSION 3.2)

set(samples
        child)
add_executable(child child.c)
foreach(s ${samples})
    target_link_libraries(${s} pthread)
endforeach(s)


