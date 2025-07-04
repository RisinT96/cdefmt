cmake_minimum_required(VERSION 3.18)
include(FetchContent)

project("cdefmt")

# Dependencies for the cdefmt c header.
FetchContent_Declare(
    preprocessor
    GIT_REPOSITORY https://github.com/boostorg/preprocessor.git
    GIT_TAG boost-1.86.0
)
FetchContent_Declare(
    vmd
    GIT_REPOSITORY https://github.com/boostorg/vmd.git
    GIT_TAG boost-1.86.0
)

FetchContent_MakeAvailable(preprocessor)
FetchContent_MakeAvailable(vmd)

# cdefmt header-only library
add_library(cdefmt INTERFACE)
target_include_directories(cdefmt INTERFACE .)
target_link_libraries(cdefmt
    INTERFACE
        Boost::preprocessor
        Boost::vmd
)

if(CMAKE_CURRENT_SOURCE_DIR STREQUAL CMAKE_SOURCE_DIR)
    set(CMAKE_EXPORT_COMPILE_COMMANDS ON)

    include(CMakeDependentOption)

    option(CDEFMT_BUILD_EXAMPLES "whether or not to build the examples" ON)

    cmake_dependent_option(
        CDEFMT_BUILD_EXAMPLE_STDOUT "build the stdout example" ON
        "CDEFMT_BUILD_EXAMPLES" OFF
    )

    cmake_dependent_option(
        CDEFMT_BUILD_EXAMPLE_STDIN "build the stdin example (rust)" ON
        "CDEFMT_BUILD_EXAMPLES" OFF
    )

    if(CDEFMT_BUILD_EXAMPLE_STDOUT)
        add_subdirectory("examples/stdout")
    endif()

    if(CDEFMT_BUILD_EXAMPLE_STDIN)
        FetchContent_Declare(
            Corrosion
            GIT_REPOSITORY https://github.com/corrosion-rs/corrosion.git
            GIT_TAG v0.5
        )
        FetchContent_MakeAvailable(Corrosion)

        corrosion_import_crate(
            MANIFEST_PATH
                Cargo.toml
            CRATES
                stdin
        )
    endif()
endif()
