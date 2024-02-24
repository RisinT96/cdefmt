cmake_minimum_required(VERSION 3.8)

project("cdefmt-example"
    LANGUAGES
        C
)

add_executable(cdefmt-example
    main.c
)

target_link_libraries(cdefmt-example
    PRIVATE
        cdefmt
)

add_custom_command(
    OUTPUT  ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
    DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/generate_linkerscript.sh
    COMMAND ${CMAKE_CURRENT_SOURCE_DIR}/generate_linkerscript.sh ${CMAKE_LINKER} ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
    COMMENT Generates linkerscript with custom sections
)
add_custom_target(linkerscript DEPENDS ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld)


set_target_properties(cdefmt-example
    PROPERTIES
        LINK_DEPENDS
            ${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
)

target_link_options(cdefmt-example
    PRIVATE
        -T${CMAKE_CURRENT_BINARY_DIR}/linkerscript.ld
)
