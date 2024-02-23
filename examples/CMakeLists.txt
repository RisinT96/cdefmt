cmake_minimum_required(VERSION 3.8)

project("cdefmt-examples"
    LANGUAGES C
)

add_executable(cdefmt-example
    main.c
)

target_link_libraries(cdefmt-example
    PRIVATE
        cdefmt
)
