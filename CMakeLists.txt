cmake_minimum_required(VERSION 3.18)
include(FetchContent)

project("cdefmt")

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.4
)
FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH Cargo.toml)

add_library(cdefmt INTERFACE)
target_include_directories(cdefmt INTERFACE .)

add_subdirectory("examples/stdout")
