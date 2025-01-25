#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/stat.h>

#include "cdefmt/include/cdefmt.h"

typedef struct some_struct {
  uint64_t a;
  uint32_t b;
  uint16_t c;
} some_struct_t;

typedef struct some_padded_struct {
  uint16_t a;
  uint32_t b;
  uint64_t c;
} some_padded_struct_t;

typedef struct __attribute__((packed)) some_packed_struct {
  uint16_t a;
  uint32_t b;
  uint64_t c;
} some_packed_struct_t;

typedef enum unsigned_enum {
  UNSIGNED_ENUM_1,
  UNSIGNED_ENUM_2,
  UNSIGNED_ENUM_3,
  UNSIGNED_ENUM_4,
  UNSIGNED_ENUM_5,
} unsigned_enum_t;

typedef enum signed_enum {
  SIGNED_ENUM_1 = 3,
  SIGNED_ENUM_2 = 2,
  SIGNED_ENUM_3 = 0,
  SIGNED_ENUM_4 = -1,
  SIGNED_ENUM_5 = INT16_MIN,
} signed_enum_t;

CDEFMT_GENERATE_INIT()

int main(int argc, char* cargv[]) {
  if (cdefmt_init()) {
    return 1;
  }

  // Different log levels:
  CDEFMT_ERROR("This is an error log.");
  CDEFMT_WARNING("This is a warning log.");
  CDEFMT_INFO("This is an info log.");
  CDEFMT_DEBUG("This is a debug log.");
  CDEFMT_VERBOSE("This is a verbose log.");

  CDEFMT_INFO("Escaped braces {{ }} } {{{{");
  CDEFMT_INFO("No closing brace { some text");

  // Different types
  bool some_bool = true;
  uint8_t some_u8 = 123;
  uint16_t some_u16 = 12345;
  uint32_t some_u32 = 1234567890;
  uint64_t some_u64 = 1234567890123456789;
  int8_t some_i8 = -123;
  int16_t some_i16 = -12345;
  int32_t some_i32 = -1234567890;
  int64_t some_i64 = -1234567890123456789;
  float some_f32 = 123.4567890123456789f;
  double some_f64 = 123.4567890123456789;

  CDEFMT_INFO("bool: [{}]", some_bool);
  CDEFMT_INFO("u8:   [{}]", some_u8);
  CDEFMT_INFO("u16:  [{}]", some_u16);
  CDEFMT_INFO("u32:  [{}]", some_u32);
  CDEFMT_INFO("u64:  [{}]", some_u64);
  CDEFMT_INFO("i8:   [{}]", some_i8);
  CDEFMT_INFO("i16:  [{}]", some_i16);
  CDEFMT_INFO("i32:  [{}]", some_i32);
  CDEFMT_INFO("i64:  [{}]", some_i64);
  CDEFMT_INFO("f32:  [{}]", some_f32);
  CDEFMT_INFO("f64:  [{}]", some_f64);

  // Format hints
  CDEFMT_INFO("no formatting  [{}]", some_u32);
  CDEFMT_INFO("width          [{:20}]", some_u32);
  CDEFMT_INFO("zero pad       [{:020}]", some_u32);

  CDEFMT_INFO("width align left   [{:<40}]", some_u32);
  CDEFMT_INFO("width align center [{:^40}]", some_u32);
  CDEFMT_INFO("width align right  [{:>40}]", some_u32);

  CDEFMT_INFO("no sign  [{:11}]", some_u32);
  CDEFMT_INFO("sign     [{:+11}]", some_u32);
  CDEFMT_INFO("negative [{:+11}]", some_i32);

  CDEFMT_INFO("Float precision [{:.3}] vs [{:<18}]", some_f32);
  CDEFMT_INFO("Float precision [{:.3}] vs [{:<18}]", some_f64);

  CDEFMT_INFO("Binary   [{:#40b}]", some_u32);
  CDEFMT_INFO("LowerExp [{:#40e}]", some_u32);
  CDEFMT_INFO("LowerHex [{:#40x}]", some_u32);
  CDEFMT_INFO("Octal    [{:#40o}]", some_u32);
  CDEFMT_INFO("Pointer  [{:#40p}]", some_u32);
  CDEFMT_INFO("UpperExp [{:#40E}]", some_u32);
  CDEFMT_INFO("UpperHex [{:#40X}]", some_u32);
  // Different structs
  some_struct_t some_struct_typedefd = {
      .a = 1234567890123456789,
      .b = 1234567890,
      .c = 12345,
  };
  struct some_struct some_struct = {
      .a = 1234567890123456789,
      .b = 1234567890,
      .c = 12345,
  };
  some_padded_struct_t some_padded_struct_typedefd = {
      .a = 12345,
      .b = 1234567890,
      .c = 1234567890123456789,
  };
  struct some_padded_struct some_padded_struct = {
      .a = 12345,
      .b = 1234567890,
      .c = 1234567890123456789,
  };
  some_packed_struct_t some_packed_struct_typedefd = {
      .a = 12345,
      .b = 1234567890,
      .c = 1234567890123456789,
  };
  struct some_packed_struct some_packed_struct = {
      .a = 12345,
      .b = 1234567890,
      .c = 1234567890123456789,
  };

  CDEFMT_INFO("some struct typedef'd:        {}", some_struct_typedefd);
  CDEFMT_INFO("some struct:                  {}", some_struct);
  CDEFMT_INFO("some padded struct typedef'd: {}", some_padded_struct_typedefd);
  CDEFMT_INFO("some padded struct:           {}", some_padded_struct);
  CDEFMT_INFO("some packed struct typedef'd: {}", some_packed_struct_typedefd);
  CDEFMT_INFO("some packed struct:           {}", some_packed_struct);
  CDEFMT_INFO("some struct alternate:        {:#}", some_struct);

  // Different enums
  enum unsigned_enum some_unsigned_enum = UNSIGNED_ENUM_5;
  enum signed_enum some_signed_enum = SIGNED_ENUM_5;
  enum unsigned_enum other_unsigned_enum = UNSIGNED_ENUM_3;
  enum signed_enum other_signed_enum = SIGNED_ENUM_3;
  CDEFMT_INFO("some unsigned enum:        [{}]", some_unsigned_enum);
  CDEFMT_INFO("other unsigned enum:       [{}]", other_unsigned_enum);
  CDEFMT_INFO("some signed enum:          [{}]", some_signed_enum);
  CDEFMT_INFO("other signed enum:         [{}]", other_signed_enum);

  // Arrays
  uint8_t u8_array[] = {1, 2, 3, 4, 5};
  CDEFMT_INFO("u8 array: {}", u8_array);

  // Up to 8 arguments
  CDEFMT_INFO("no args []");
  CDEFMT_INFO("1 arg:  [{}]", some_bool);
  CDEFMT_INFO("2 args: [{}, {}]", some_bool, some_i8);
  CDEFMT_INFO("3 args: [{}, {}, {}]", some_bool, some_i8, some_u8);
  CDEFMT_INFO("4 args: [{}, {}, {}, {}]", some_bool, some_i8, some_u8, some_f32);
  CDEFMT_INFO("5 args: [{}, {}, {}, {}, {}]", some_bool, some_i8, some_u8, some_f32, some_f64);
  CDEFMT_INFO("6 args: [{}, {}, {}, {}, {}, {}]", some_bool, some_i8, some_u8, some_f32, some_f64,
              some_packed_struct);
  CDEFMT_INFO("7 args: [{}, {}, {}, {}, {}, {}, {}]", some_bool, some_i8, some_u8, some_f32,
              some_f64, some_packed_struct, some_i64);
  CDEFMT_INFO("8 args: [{}, {}, {}, {}, {}, {}, {}, {}]", some_bool, some_i8, some_u8, some_f32,
              some_f64, some_packed_struct, some_i64, some_unsigned_enum);

  // Handle user error
  CDEFMT_INFO("HAHA I LIED! gave you no args at all! [{}, {}, {}, {}]");
  CDEFMT_INFO("HAHA I LIED! gave you less args than in format string! [{}, {}, {}, {}]", some_bool,
              some_signed_enum, u8_array);

  // Of course can print same log with different values.
  for (size_t i = 0; i < 10; i++) {
    CDEFMT_INFO("Iteration {}", i);
  }

  char some_string[] = "this is some string";

  // Quotes have to be double escaped.
  CDEFMT_INFO("Some string: \\\"{:s}\\\"", some_string);

  CDEFMT_INFO("Named parameters: {some_f32} {some_struct.b} {1} {some_u16} {}", some_bool, some_u16,
              some_f32, some_struct.b);

  CDEFMT_INFO("Wrong named parameters: {asome_f32} {some_struct.ba} {1} {some_u16} {}", some_bool, some_u16,
              some_f32, some_struct.b);

  return 0;
}

void cdefmt_log(const void* log, size_t size, enum cdefmt_level level) {
  struct stat stat;

  fstat(1, &stat);

  if (S_ISFIFO(stat.st_mode)) {
    // Write raw binary data
    fwrite(&size, sizeof(size), 1, stdout);
    fwrite(log, size, 1, stdout);
    return;
  }

  // For regular stdout we pretty print.
  printf("level: %u, id: %#010lx, size: %-3zu data: [", level, ((const uintptr_t*)log)[0], size);

  for (size_t i = 0; i < size; i++) {
    printf("%02x", ((const uint8_t*)log)[i]);

    if (i + 1 < size) {
      printf(", ");
    }
  }

  printf("]\n");
}
