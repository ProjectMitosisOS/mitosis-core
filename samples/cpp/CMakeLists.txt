project(simple_cpp_examples)
ADD_DEFINITIONS(-std=c++17)
cmake_minimum_required(VERSION 3.2)

set(samples
        child parent)
add_executable(child child.cc)
add_executable(parent parent.cc)
foreach(s ${samples})
    target_link_libraries(${s} pthread)
endforeach(s)


