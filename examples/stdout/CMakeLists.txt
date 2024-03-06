add_executable(example-stdout
    main.c
    linkerscript.ld
)

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

set_target_properties(example-stdout
    PROPERTIES
        LINK_DEPENDS
            ${CMAKE_CURRENT_SOURCE_DIR}/linkerscript.ld
)
