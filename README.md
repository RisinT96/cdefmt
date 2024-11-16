# `cdefmt`

`cdefmt` ("c de format", short for "c deferred formatting") is a highly efficient logging framework that targets resource-constrained devices, like microcontrollers.

Inspired by <https://github.com/knurling-rs/defmt/>

# What is this?

The idea is simple, we want to spend as little time as possible on creating logs, and have them take up as little space as possible.

`cdefmt` achieves that through a multi phased approach:
1. Encode as much information as possible at compile time<br>
   Done via the macros in `cdefmt/include/cdefmt.h` and some modifications to the compilation flags/linkerscript.<br>
   All of the log strings, parameter format and size information are encoded at compile time into the binary, these can be stripped from the shipped binary.
2. Have minimal information encoded into each log - only an ID and the raw values of the arguments.
3. Defer the formatting logic to post processing<br>
   Performed by the rust library [cdefmt-decoder](decoder/), using information from the originally compiled binary to parse the ID and arguments into a full string.

For more technical details refer to [Technical Details](#technical-details)

# Usage

Before using the logging framework, there are some [setup](#setup) steps necessary to integrate the encoding logic into your compiled binary.

Once that's done, you can use the `cdefmt-decoder` library to decode your logs.

## Example

A working example is provided under:
*   [stdout](examples/stdout/) generates the logs and prints them to `stdout`<br>
    If `stdout` is a `FIFO` (aka shell pipe `|`), the output format changes to a more compact, easy to parse, but not human understandable format.
*   [stdin](examples/stdin/) takes the logs from it's `stdin` and, using the original binary file, parses them into proper logs.<br>
    It expects the input to be formatted in the same way as `stdout` outputs in `FIFO` mode.

The easiest way to run the example would be to build the project using cmake:
```bash
cmake -S . -B build
cmake --build build/
```

Then run `example-stdout` and pipe it's stdout into the `example-stdin`, while providing `example-stdin` a path to the originally compiled binary of `example-stdout`:

```bash
build/examples/stdout/example-stdout | build/stdin --elf build/examples/stdout/example-stdout
```

To further complicate matters, it's possible to strip `example-stdout` and run the stripped binary, logs will still work:
```bash
# Remove all symbols and debugging information from example-stdout
strip build/examples/stdout/example-stdout -o build/examples/stdout/example-stdout-stripped --strip-all
# Remove `.cdefmt` section from the stripped binary
objcopy --remove-section .cdefmt build/examples/stdout/example-stdout-stripped
# Run the example (note that the parser still needs the original un-stripped binary)
build/examples/stdout/example-stdout-stripped | build/stdin --elf build/examples/stdout/example-stdout
```

This means that it's possible to ship stripped binaries, while using the originals to parse the logs.

The stripping process removes the following from the binary:
1. log strings
2. debugging information
3. symbol table

This has 2 advantages:
1. reduced size
2. obfuscation

## Setup

### c

1.  cdefmt encodes the log strings along with some metadata into a special section in the elf binary, we need to modify the project's linker script to generate that section:<br>
    Add the cdefmt metadata section to the end of the linker script (or right before `/DISCARD/` if it exists):
    ```
    /* CDEFMT: log metadata section */
    .cdefmt 0 (INFO) : {
        KEEP(*(.cdefmt.init .cdefmt.init.*))
        . = . + 8;
        KEEP(*(.cdefmt .cdefmt.*))
    }
    ```
2.  cdefmt uses the build-id to uniqely identify an elf file, and validate the compatibility of the parsed logs with the supplied elf:<br>
    Update (or add if it doesn't exist) the `.note.gnu.build-id` section:
    ```
    .note.gnu.build-id : {
        PROVIDE(__cdefmt_build_id = .);
        *(.note.gnu.build-id)
    }
    ```
3.  Compile with the following flags:
    * `-g`              - cdefmt uses debugging information to parse log arguments.
    * `-Wl,--build-id`  - link the build-id into the binary.
4.  Add the header [`cdefmt.h`](cdefmt/include/cdefmt.h) to your project.
5.  Include the header wherever you want to use it.
6.  Implement [cdefmt_log](cdefmt/include/cdefmt.h#L47) to forward the logs to your logging backend.
7.  Call [CDEFMT_GENERATE_INIT](cdefmt/include/cdefmt.h#L21) outside of a function scope.
8.  Call [cdefmt_init](cdefmt/include/cdefmt.h#L20) after your logging backend is initialized and `cdefmt_log` can be safely called.
9.  Enjoy.

See [example project](examples/stdout/) for reference.

# Technical Details
TBD

# License
*   MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
