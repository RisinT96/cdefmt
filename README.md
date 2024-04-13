# `cdefmt`

`cdefmt` ("c de format", short for "c deferred formatting") is a highly efficient logging framework that targets resource-constrained devices, like microcontrollers.

Inspired by <https://github.com/knurling-rs/defmt/>

# Setup

## c
1.  Add cdefmt modifications to the linkerscript:
    1.  Update/add `.note.gnu.build-id` section to define the necessary memory locations:
        ```
        .note.gnu.build-id : {
            PROVIDE(__cdefmt_build_id = .);
            *(.note.gnu.build-id)
        }
        ```
    2.  Add the cdefmt metadata section to the end of the linker script (or right before
        `/DISCARD/` if it exists):
        ```
        /* CDEFMT: log metadata section */
        .cdefmt 0 (INFO) : {
            KEEP(*(.cdefmt.init .cdefmt.init.*))
            . = . + 8;
            KEEP(*(.cdefmt .cdefmt.*))
        }
        ```
2.  Make sure that you compile with the following flags:
    * `-g`              - need debugging information in order to parse log arguments.
    * `-Wl,--build-id`  - link build-id into the binary to verify elf compatibility on init.
3.  Add the header [`cdefmt.h`](cdefmt/include/cdefmt.h) to your project.
4.  Include the header wherever you want to use it.
5.  Implement `cdefmt_log` in your project.
6.  Call `CDEFMT_GENERATE_INIT()` somewhere in your main file.
7.  Call `cdefmt_init()` after your logging backend is initialized and `cdefmt_log` can be safely called.
8.  Enjoy.

## Rust
TBD

# Usage

Basically `cdefmt_log` should store/write `log` into the log sink, then these bytes should be
provided to the parser as-is, accompanied with the elf binary.

See:
*   [stdout](examples/stdout/) for generating logs.
*   [stdin](examples/stdin/) for parsing the logs.

The easiest way to run the example would be to build the project using cmake:
```bash
cmake -S . -B build
cmake --build build/
```

Then run the stdout example and pipe it's stdout into the stdin example:

```bash
build/debug/examples/stdout/example-stdout | build/debug/stdin --elf build/debug/examples/stdout/example-stdout
```

# License
*   MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
