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
    - [4.2.1. Dependencies](#421-dependencies)
    - [4.2.2. Integration](#422-integration)
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

### 4.2.1. Dependencies

`cdefmt` is a header only library, implemented using mostly macros, however, it's dependent on the boost preprocessor and VMD libraries.
These are header only libraries compatible with c and c++.

If working with CMake, and linking with the `cdefmt` library, you should automatically get the required dependencies.
Otherwise you'll have to provide your own, using your build system.

### 4.2.2. Integration

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
2.  cdefmt uses the GNU build-id to uniquely identify an elf file, and validate the compatibility of the parsed logs with the supplied elf:<br>
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
4.  Copy the header [`cdefmt_config.h`](cdefmt/config/cdefmt_config.h) to your project, place it in a `config` directory, and make sure that your project has it in it's include path.

    Example project structure:
    ```
    project_root
    ├── src
    │   └── main.c
    └── include
        ├── config/cdefmt_config.h
        └── some_header.h
    ```
    Make sure that `proect_root/include` is your project's include path, as `cdefmt.h` is looking for a `config/cdefmt_config.h` include file.
5.  Update your copy of `cdefmt_config.h` to reflect your desired configuration.
6.  Include [`cdefmt.h`](cdefmt/include/cdefmt.h) to use the logger.
8.  Implement [cdefmt_log](cdefmt/include/cdefmt.h#L59) to forward the logs to your logging backend.
9.  Call [CDEFMT_GENERATE_INIT](cdefmt/include/cdefmt.h#L27) outside of a function scope.
10. Call [cdefmt_init](cdefmt/include/cdefmt.h#L26) after your logging backend is initialized and `cdefmt_log` can be safely called.
11. Enjoy.

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
2       static const struct __attribute__((packed)) {
3           uint32_t version;
4           uint32_t counter;
5           uint32_t line;
6           uint32_t file_len;
7           uint32_t fmt_len;
8           uint32_t names_len;
9           uint8_t level;
10          char file[sizeof("/root/git/cdefmt/examples/stdout/main.c")];
11          char fmt[sizeof("Example log {} {:x}")];
12          struct __attribute__((packed)) {
13              struct __attribute__((packed)) {
14                  uint32_t len;
15                  char name[sizeof("argument_a")];
16              } n0;
17              struct __attribute__((packed)) {
18                  uint32_t len;
19                  char name[sizeof("arg_b")];
20              } n1;
21          } names;
22      } cdefmt_log_metadata50 __attribute__((section(".cdefmt.metadata"))) = {
23          .version = 1,
24          .counter = (50),
25          .line = (174),
26          .file_len = (sizeof("/root/git/cdefmt/examples/stdout/main.c")),
27          .fmt_len = (sizeof("Example log {} {:x}")),
28          .names_len = (2),
29          .level = (2),
30          .file = ("/root/git/cdefmt/examples/stdout/main.c"),
31          .fmt = ("Example log {} {:x}"),
32          .names = {
33              .n0 = {
34                  .len = sizeof("argument_a"),
35                  .name = ("argument_a"),
36              },
37              .n1 = {
38                  .len = sizeof("arg_b"),
39                  .name = ("arg_b"),
40              },
41          },
42      };
43      struct __attribute__((packed)) cdefmt_log_args_t50 {
44          const void* log_id;
45          __typeof__(argument_a) arg0;
46          __typeof__(arg_b) arg1;
47          uint8_t dynamic_data[128];
48      };
49      size_t cdefmt_dynamic_size = 0;
50      struct cdefmt_log_args_t50* cdefmt_log_args50 = (&(struct cdefmt_log_args_t50){0});
51      cdefmt_log_args50->log_id = &(cdefmt_log_metadata50);
52      size_t cdefmt_dynamic_offset = 0;
53      do {
54          memcpy(&(cdefmt_log_args50->arg0), &(argument_a), sizeof(cdefmt_log_args50->arg0));
55      } while (0);
56      do {
57          memcpy(&(cdefmt_log_args50->arg1), &(arg_b), sizeof(cdefmt_log_args50->arg1));
58      } while (0);
59      cdefmt_log(cdefmt_log_args50, (sizeof(*cdefmt_log_args50) - (128 - cdefmt_dynamic_offset)), 2);
60      ;
61    } while (0);
```
Let's go over each line and explain what's happening there:

1.  Standard `do { ... } while (0)` macro mechanism:
    *   Creates scope for local variables
    *   Requires user to add a semicolon after the macro call
    *   Protects against `if (...) macro_call(); else ...` issues
2.  Define a stract type:
    * This struct will hold the metadata for this log.
    * The struct is packed to minimize the metadata size.
    * The struct is static, allowing the compiler to assign it a permanent address, rather than placing it on the stack.
    * The struct is const to ensure immutability.
3.  Define the metadata version (for future compatibility).
4.  Define a unique counter, the value is unique per log in each compilation unit.
    There might be multiple logs with the same value in different compilation units.
    This value will be used to identify the log when decoding the logs.
5.  Define the line number where the log is defined.
6.  Define the length of the filename string - used when parsing the metadata structure.
7.  Define the length of the format string - used when parsing the metadata structure.
8.  Define the amount of named arguments profided - used when parsing the metadata structure.
9.  Define the level of the log.
10. Define the filename where the log is defined - this is used later to find the compilation unit.
11. Define the format string.
12. Define a sub-struct - containing the names of the arguments.
13. Define a sub struct - containing the name of the first argument.
14. Define the length of the first argument's name.
15. Define the first argument's name.
16. Close the struct.
17. Define a sub struct - containing the name of the second argument.
18. Define the length of the second argument's name.
19. Define the second argument's name.
20. Close the struct.
21. Close the names struct.
22. Name the metadata variable, and place it into the `.cdefmt.metadata` section.
    *   Notice how the variable's name ends with `50`, the same value that will be assigned to the `counter` field.
23. Assign the version, currently it's 1.
24. Assign the counter value.
25. Assign the log line.
26. Assign the filename length.
27. Assign the format string length.
28. Assign the amount of arguments.
29. Assign the log's level (2 = INFO).
30. Assign the filename.
31. Assign the format string.
32. Assign the names of the arguments.
33. Assign the name of the first argument.
34. Assign the length of the name.
35. Assign the name itself.
36. Closing scope.
37. Assign the name of the second argumnet.
38. Assign the length of the name.
39. Assign the name itself.
40. Closing scope.
41. Closing scope.
42. Closing scope.
43. Define a struct for holding the log information that will be sent over the wire.
    *   The struct is packed to save space.
44. Log ID - pointer to the log metadata structure.
45. First argument.
46. Second argument.
47. Array for holding dynamically sized arguments.
48. Closing scope.
49. Assign dynamic size to 0, as we don't have any dynamic arguments.
50. Create an instance of the log args structure.
51. Assign the log_id, to the address of the metadata structure.
52. Assign the dynamic offset to 0, we can increment it later if there's any dynamic arguments.
53. See 1.
54. Copy the first argument into the log args structure.
55. See 1.
56. See 1.
57. Copy the second argument into the log args structure.
58. See 1.
59. Send the log args structure to the log function.
    *   Notice that the length should calculate the size of the log args structure, which in this case is `sizeof(log_args_t)` minus the entire dynamic data as there's no dynamic data.
60. Placeholder in case user opts in for a static/heap log buffer rather than a stack based one.
61. See 1

>  ⚠️ Notice how we don't access the log metadata structure at any time during runtime, only its address.

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

When the project is compiled and linked, each log metadata struct is assigned a unique address by the linker, this address can be used as a unique ID to identify the log.<br>
The compilation process embeds these addresses into the places that reference the log metadata, because, being marked `INFO` and `static`, they don't depend on any runtime linkage, and are hardcoded by the linker, this means **we can completely remove the `.cdefmt` section from the shipped binary**, without affecting any of the functionality.


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
3.  Find the log metadata in the `.cdefmt` section, and parse the binary structure.
4.  Use the counter and source file to find the debugging information describing the log's arguments.
5.  Parse the remaining fields of the log args structure.
6.  Format the log string with the parsed arguments.
7.  Profit :D

# 6. License
*   MIT license ([LICENSE](LICENSE) or http://opensource.org/licenses/MIT)
