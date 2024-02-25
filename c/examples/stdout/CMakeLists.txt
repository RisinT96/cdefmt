add_executable(example-stdout
    main.c
)

target_link_libraries(example-stdout
    PRIVATE
        cdefmt
)

target_link_options(example-stdout
    PRIVATE
        -T${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
)

add_dependencies(example-stdout linkerscript)
set_target_properties(example-stdout
    PROPERTIES
        LINK_DEPENDS
            ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
)

add_custom_target(linkerscript DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld)
add_custom_command(
    OUTPUT  ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/generate_linkerscript.sh
    COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/generate_linkerscript.sh ${CMAKE_LINKER} ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
    COMMENT Generates linkerscript with custom sections
)

