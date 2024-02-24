cmake_minimum_required(VERSION 3.18)
include(FetchContent)

project("cdefmt"
    LANGUAGES C
)

add_library(cdefmt INTERFACE)
target_include_directories(cdefmt INTERFACE .)

FetchContent_Declare(
    Corrosion
    GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
    GIT_TAG v0.4
)
FetchContent_MakeAvailable(Corrosion)

corrosion_import_crate(MANIFEST_PATH parser/Cargo.toml)

add_subdirectory("examples")

