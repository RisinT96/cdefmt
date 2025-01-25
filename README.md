# 1. `cdefmt`

`cdefmt` ("c de format", short for "c deferred formatting") is a highly efficient logging framework that targets resource-constrained devices, like microcontrollers.

Inspired by <https://github.com/knurling-rs/defmt/>

# 2. Table of Contents

- [1. `cdefmt`](#1-cdefmt)
- [2. Table of Contents](#2-table-of-contents)
- [3. What is this?](#3-what-is-this)
- [4. Usage](#4-usage)
  - [4.1. Example](#41-example)
  - [4.2. Setup](#42-setup)
    - [4.2.1. c](#421-c)
- [5. Technical Details](#5-technical-details)
  - [5.1. Generation](#51-generation)
  - [5.2. Metadata](#52-metadata)
    - [5.2.1. linker script `.cdefmt` section](#521-linker-script-cdefmt-section)
    - [5.2.2. Debugging information](#522-debugging-information)
  - [5.3. Parsing](#53-parsing)
- [6. License](#6-license)

# 3. What is this?

The idea is simple, we want to spend as little time as possible on creating logs, and have them take up as little space as possible.

`cdefmt` achieves that through a multi phased approach:
1. Encode as much information as possible at compile time<br>
   Done via the macros in `cdefmt/include/cdefmt.h` and some modifications to the compilation flags/linkerscript.<br>
   All of the log strings, parameter format and size information are encoded at compile time into the binary, these can be stripped from the shipped binary.
2. Have minimal information encoded into each log - only an ID and the raw values of the arguments.
3. Defer the formatting logic to post processing<br>
   Performed by the rust library [cdefmt-decoder](decoder/), using information from the originally compiled binary to parse the ID and arguments into a full string.

For more technical details refer to [Technical Details](#technical-details)

# 4. Usage

Before using the logging framework, there are some [setup](#setup) steps necessary to integrate the encoding logic into your compiled binary.

Once that's done, you can use the `cdefmt-decoder` library to decode your logs.

## 4.1. Example

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

## 4.2. Setup

### 4.2.1. c

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

# 5. Technical Details

We'll follow the process of generating and parsing/formatting a single log.

## 5.1. Generation

Starting off with a simple log taking two arguments:

```c
CDEFMT_INFO("Example log {} {:x}", argument_a, arg_b);
```

This macro is expanded by the preprocessor into

```c
1   do {
2       const static __attribute__((section(".cdefmt"))) char cdefmt_log_string2[] = "{""\"version\":""1"",""\"counter\":""2"",""\"level\":""2"",""\"file\":\"""/root/git/cdefmt/examples/stdout/main.c""\",""\"line\":""52"",""\"message\":\"""Example log {} {:x}""\",""\"names\": [""\"""argument_a""\""",""\"""arg_b""\"""]""}";
3       struct __attribute__((packed)) cdefmt_log_args_t2 {
4           const char* log_id;
5           __typeof__(argument_a) arg0;
6           __typeof__(arg_b) arg1;
7       };
8       struct cdefmt_log_args_t2 cdefmt_log_args2 = {
9           .log_id = cdefmt_log_string2,
10      };
11      do {
12          memcpy((&cdefmt_log_args2.arg0), &(argument_a), sizeof(cdefmt_log_args2.arg0));
13      } while (0);
14      do {
15          memcpy((&cdefmt_log_args2.arg1), &(arg_b), sizeof(cdefmt_log_args2.arg1));
16      } while (0);
17      cdefmt_log(&cdefmt_log_args2, sizeof(cdefmt_log_args2), 2);
18  } while (0);
```
Let's go over each line and explain what's happening there:

1.  Standard `do { ... } while (0)` macro mechanism:
    *   Creates scope for local variables
    *   Requires user to add a semicolon after the macro call
    *   Protects against `if (...) macro_call(); else ...` issues
2.  We have a few things going on here, let's break them down:
    *   `const static char cdefmt_log_string2[]`:<br>
        We're creating a const static string that will hold the actual log string along with some metadata.<br>
        Making it `static` allows the compiler to allocate a permanent address for this string, this will come into play later 😉.<br>
        > Notice how the string's name ends with `2`, the same value as in the `counter` field in the json.
    *   `__attribute__((section(".cdefmt")))`:<br>
        Tell the linker to place the string into the special `.cdefmt` section in the elf.<br>
        This allows us to separate all the log strings into a separate section, which can later be stripped from the elf, removing any trace of the strings from the binary.
    *   Now comes the string itself, it's generated using some macro magic and depends on a known c feature - any consecutive string literals are merged into a single string literal by the compiler.<br>
        The resulting string is actually a simple json, let me format it for readability:
        ```json
        {
            "version": 1,
            "counter": 2,
            "level": 2,
            "file": "/root/git/cdefmt/examples/stdout/main.c",
            "line": 52,
            "message": "Example log {} {:x}",
            "names": ["argument_a", "arg_b"]
        }
        ```
        | field | explanation |
        | ----- | ----------- |
        | version | json schema version |
        | counter | unique counter identifying this log, the uniqueness is per compilation unit (`.c` file), so logs from  |different files can repeat the value.
        | level | the log level (verbose, debug, etc...) |
        | file | the file where this log came from |
        | line | the code line the log came from |
        | message | the user provided format string |
        | names | the argument names, in the same order as the user provided them |
3.  We're creating a new type which will uniquely identify this log and it's arguments.<br>
    The struct is `packed` to remove any padding, and make it as small as possible.<br>
    > Notice how the struct's name ends with `2`, the same value as in the `counter` field in the json.
4.  The first value in the struct is the log id.
5.  The second value in the struct is a copy of the first argument.
6.  The third value in the struct is a copy of the second argument.
7.  Closing the type definition.
8.  Instantiate a variable of the newly defined type.<br>
    > Notice how `2` appears here again in all the type/variable names.
9.  Assign the address of the log string into the `log_id` field.<br>
    We'll be using this address as the log's unique identifier, the parser will be able to extract the log string from the original elf using this value.
10. Closing the initializer.
11. See 1.
12. Copy the value of the first argument into the log structure.
13. See 1.
14. See 1.
15. Copy the value of the second argument into the log structure.
16. See 1.
17. Send the log into the user implemented logging backend
    | argument | explanation |
    | -------- | ----------- |
    | &cdefmt_log_args2 | pointer to the log structure |
    | sizeof(cdefmt_log_args2) | size of the log structure |
    | 2 | log level (can be used for runtime filtering by the user) |
    From the user's point of view, he's receiving a binary blob, it's the user's responsibility to store this blob, and then use the parsing library to parse this blob back into the log string.
18. See 1.

>  ⚠️ Notice how we don't access the log string at any time during runtime, only it's address.

## 5.2. Metadata

Let's look at how the necessary metadata is stored in the binary.

### 5.2.1. linker script `.cdefmt` section

As we've seen in the previous section, the log metadata is stored in a special `.cdefmt` section, this section is manually defined in the linker script:
```
1.  /* CDEFMT: log metadata section */
2.  .cdefmt 0 (INFO) : {
3.      KEEP(*(.cdefmt.init .cdefmt.init.*))
4.      . = . + 8;
5.      KEEP(*(.cdefmt .cdefmt.*))
6.  }
```

1.  Comment
2.  Define a new section:
    | parameter | explanation |
    | --------- | ----------- |
    | `.cdefmt` | Section name |
    | `0` | Place the section at address 0.<br>All the logs we insert into this section will start from address 0 |
    | `(INFO)` | Set the section type as not allocatable, so that no memory is allocated for the section when the program is run. |
3.  Put the `.cdefmt.init` inputs into this section, this comes first as we want the init log to be uniquely placed at address 0.
4.  Advance the location by 8, this ensures that even if the init log wasn't compiled, none of the other logs will ever have id 0.
5.  Put the `.cdefmt` inputs into this section, this is where all the log strings/metadata will be stored.

To gain a better understanding I recommend reading through the [binutils ld docs](https://sourceware.org/binutils/docs/ld/Scripts.html).

When the project is compiled and linked, each log string is assigned a unique address by the linker, this address can be used as a unique ID to identify the log.<br>
The compilation process embeds these addresses into the places that reference the log strings, because, being marked `INFO` and `static`, they don't depend on any runtime linkage, and are hardcoded by the linker, this means **we can completely remove the `.cdefmt` section from the shipped binary**, without affecting any of the functionality.


Now we know where the metadata strings are stored, and how we can extract them using the log ID, however we still don't know how to parse the log's arguments, as they're in a packed struct and can have any size, type or structure.<br>
The next section will discuss exactly that.

### 5.2.2. Debugging information

As we've seen previously, the log's arguments are copied into a packed struct, we need to know it's structure to be able to parse the binary blob back into usable data.

This is where debugging information comes in handy.<br>
One of the requirements of this framework is compiling the code with `-g`, this instructs the compiler/linker to generate debugging information and embed it into the elf.

ELF uses the [DWARF](https://dwarfstd.org/index.html) debugging format to store the debugging information, we can use that to our advantage.

Short reminder from the [Generation](#generation) section, for each log we're doing 3 things that are useful here:
1.  Defining a new type: `struct cdefmt_log_args_t<counter>`.
2.  Inserting `counter` into the log's metadata string.
3.  Inserting the source file path into the log's metadata string.

The compiler will generate debugging information that describes this new type, and will embed it into the binary.

Using the counter value, and the source file path, we can find the specific compilation unit that defines the structure of the log args struct, there we can find the debugging information to figure out how to correctly parse the log's arguments from a binary blob.

Since the debugging information is only needed at parsing time, it can be stripped from the shipped binary.

## 5.3. Parsing

All that's left is to parse the binary blobs generated by the logger:

1.  Load the **original, unstripped** binary - there we find the `.cdefmt` section and the debugging information.
2.  Extract the log id from the binary blob.<br>
    Since we have the elf, we know the size of a pointer in the target, so we know the size of the log id.
3.  Find the log metadata in the `.cdefmt` section, and parse the json.
4.  Use the counter and source file to find the debugging information describing the log's arguments.
5.  Parse the remaining fields of the log args structure.
6.  Format the log string with the parsed arguments.
7.  Profit :D

# 6. License
*   MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
