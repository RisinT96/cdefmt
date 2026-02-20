add_executable(example-stdout
    main.c
    linkerscript.ld
)

set_target_properties(example-stdout
    PROPERTIES
        C_STANDARD 17
        C_STANDARD_REQUIRED ON
        LINKER_LANGUAGE C
        LINK_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/linkerscript.ld
)

target_include_directories(example-stdout PRIVATE .)

target_link_libraries(example-stdout
    PRIVATE
        cdefmt
)

target_link_options(example-stdout
    PRIVATE
        -T${CMAKE_CURRENT_SOURCE_DIR}/linkerscript.ld
        -Wl,--build-id
)

target_compile_options(example-stdout
    PRIVATE
        -g
)
