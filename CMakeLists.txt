cmake_minimum_required(VERSION 3.18)
include(FetchContent)

project("cdefmt")

set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.5
)
FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH Cargo.toml)

FetchContent_Declare(
    preprocessor
    GIT_REPOSITORY https://github.com/boostorg/preprocessor.git
    GIT_TAG boost-1.86.0
)
FetchContent_MakeAvailable(preprocessor)

FetchContent_Declare(
    vmd
    GIT_REPOSITORY https://github.com/boostorg/vmd.git
    GIT_TAG boost-1.86.0
)
FetchContent_MakeAvailable(vmd)

add_library(cdefmt INTERFACE)
target_include_directories(cdefmt INTERFACE .)
target_link_libraries(cdefmt
    INTERFACE
        Boost::preprocessor
        Boost::vmd
)
target_link_libraries(cdefmt INTERFACE Boost::preprocessor)

add_subdirectory("examples/stdout")
