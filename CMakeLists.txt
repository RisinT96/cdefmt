cmake_minimum_required(VERSION 3.8)

project("cdefmt"
    LANGUAGES C
)

add_library(cdefmt INTERFACE)
target_include_directories(cdefmt INTERFACE .)

add_subdirectory("examples")
