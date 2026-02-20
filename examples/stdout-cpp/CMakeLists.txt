add_executable(example-stdout-cpp
    main.cpp
    linkerscript.ld
)

set_target_properties(example-stdout-cpp
    PROPERTIES
        CXX_STANDARD 23
        CXX_STANDARD_REQUIRED ON
        LINKER_LANGUAGE CXX
        LINK_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/linkerscript.ld
)

target_include_directories(example-stdout-cpp PRIVATE .)

target_link_libraries(example-stdout-cpp
    PRIVATE
        cdefmt
)

target_link_options(example-stdout-cpp
    PRIVATE
        -T${CMAKE_CURRENT_SOURCE_DIR}/linkerscript.ld
        -Wl,--build-id
)

target_compile_options(example-stdout-cpp
    PRIVATE
        -g
)
