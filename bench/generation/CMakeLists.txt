function(add_bench_target target_name compile_def)
    add_executable(${target_name}
        main.c
        linkerscript.ld
    )
    set_target_properties(${target_name}
        PROPERTIES
            C_STANDARD 17
            C_STANDARD_REQUIRED ON
            LINKER_LANGUAGE C
            LINK_DEPENDS ${CMAKE_CURRENT_SOURCE_DIR}/linkerscript.ld
    )
    target_include_directories(${target_name} PRIVATE .)
    target_link_libraries(${target_name} PRIVATE cdefmt)
    target_link_options(${target_name}
        PRIVATE
            -T${CMAKE_CURRENT_SOURCE_DIR}/linkerscript.ld
            -Wl,--build-id
    )
    target_compile_options(${target_name} PRIVATE -g)
    target_compile_definitions(${target_name} PRIVATE ${compile_def})
endfunction()

add_bench_target(bench-stack "CDEFMT_USE_STACK_LOG_BUFFER=1")
add_bench_target(bench-static "CDEFMT_USE_STATIC_LOG_BUFFER=1")
add_bench_target(bench-dynamic "CDEFMT_USE_DYNAMIC_LOG_BUFFER=1")
