#include <cdefmt/include/cdefmt.h>
#include <pthread.h>
#include <stdbool.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/stat.h>

#if defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER
uint8_t cdefmt_global_buffer[CDEFMT_STATIC_LOG_BUFFER_SIZE];
pthread_mutex_t cdefmt_global_buffer_lock;
#endif /* defined (CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER */

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
#if defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER
  pthread_mutex_init(&cdefmt_global_buffer_lock, NULL);
#endif /* defined (CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER */

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

  // Tons of arguments:
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

  // No arguments provided
  CDEFMT_INFO("These parameters have no arguments: [{}, {1}, {2}, {hey_bro}]");
  CDEFMT_INFO("Requested 4 parameters but provided only 3: [{}, {}, {}, {}]", some_bool,
              some_signed_enum, u8_array);

  // Dynamic values with same log
  for (size_t i = 0; i < 10; i++) {
    CDEFMT_INFO("Iteration {}", i);
  }

  char some_string[] = "this is some string";

  // Double quotes have to be escaped.
  CDEFMT_INFO("Some string: \"{:s}\"", some_string);

  char hidden_message[] = "I'm a hidden message!";
  char string_in_big_array[40 + sizeof(hidden_message)] = "this is some string";
  memcpy(string_in_big_array + 30, hidden_message, sizeof(hidden_message));
  CDEFMT_INFO("hidden message: '{:s}'", string_in_big_array);

  CDEFMT_INFO("Named parameters: {some_f32} {some_struct.b} {1} {some_u16} {}", some_bool, some_u16,
              some_f32, some_struct.b);

  CDEFMT_INFO("Wrong named parameters: {asome_f32} {some_struct.ba} {1} {some_u16} {}", some_bool,
              some_u16, some_f32, some_struct.b);

  // Dynamic strings

  char* dynamic_string = "This is a dynamic string, the size is not known at compile time.";

  CDEFMT_INFO("Dynamic string: {:s}", CDEFMT_DYNAMIC_STRING(dynamic_string));
  CDEFMT_INFO("Dynamic string (truncated): {:s}", CDEFMT_DYNAMIC_STRING_N(dynamic_string, 20));

  // Dynamic arrays
  some_struct_t* dynamic_struct = calloc(3, sizeof(some_struct_t));
  size_t struct_len = 2;

  dynamic_struct[0].a = 1;
  dynamic_struct[0].b = 2;
  dynamic_struct[0].c = 3;
  dynamic_struct[1].a = 101;
  dynamic_struct[1].b = 102;
  dynamic_struct[1].c = 103;
  dynamic_struct[2].a = 201;
  dynamic_struct[2].b = 202;
  dynamic_struct[2].c = 203;

  CDEFMT_INFO("Dynamic array: {}", CDEFMT_DYNAMIC_ARRAY(dynamic_struct, struct_len));

  struct_len++;

  CDEFMT_INFO("Dynamic array 2 : {}", CDEFMT_DYNAMIC_ARRAY(dynamic_struct, struct_len));

  CDEFMT_INFO("Dynamic array: {}, some_packed_struct: {}, dynamic string: '{:s}'",
              CDEFMT_DYNAMIC_ARRAY(dynamic_struct, struct_len), some_packed_struct,
              CDEFMT_DYNAMIC_STRING(dynamic_string));

  /* Thanks ChatGPT 😅 */
  char* really_long_string =
      "The Last Signal\n"
      "The spaceship Aurora drifted through the silent void, its systems failing one by one. \n"
      "Captain Elias sat in the dim cockpit, staring at the blinking distress beacon. It had been "
      "\n"
      "three days since the engine failure. No response. No signs of life. Just the endless "
      "black.\n"
      "He tapped the cracked screen of the communicator. Static. His crew was gone, lost to the \n"
      "malfunction that had turned Aurora into a metal coffin. He refused to die in silence.\n"
      "With the last of the ship's energy, he boosted the beacon's range and spoke:\n"
      "\"This is Captain Elias of the Aurora. If anyone is out there... I won't last much longer.\n"
      "But if you hear this, know that I was here. I existed.\"\n"
      "He leaned back, exhaling slowly. Then, just as the power flickered out—\n"
      "A response.\n"
      "\"Captain Elias, we hear you. Hold on.\"\n"
      "A smile formed as darkness embraced him. He wasn't alone after all.";

  CDEFMT_INFO("A very long dynamic string: {really_long_string:s}",
              CDEFMT_DYNAMIC_STRING(really_long_string));

#if defined(CDEFMT_USE_STACK_LOG_BUFFER) && CDEFMT_USE_STACK_LOG_BUFFER
  CDEFMT_INFO("Using stack buffer, truncated to " BOOST_PP_STRINGIZE(CDEFMT_STACK_LOG_BUFFER_DYNAMIC_SIZE_MAX) " bytes");
#endif /* defined (CDEFMT_USE_STACK_LOG_BUFFER) && CDEFMT_USE_STACK_LOG_BUFFER */

#if defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER
  CDEFMT_INFO("Using static buffer, truncated to  " BOOST_PP_STRINGIZE(CDEFMT_STATIC_LOG_BUFFER_SIZE) " bytes");
#endif /* defined (CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER */

#if defined(CDEFMT_USE_DYNAMIC_LOG_BUFFER) && CDEFMT_USE_DYNAMIC_LOG_BUFFER
  CDEFMT_INFO("Using dynamic buffer, no truncation");
#endif /* defined (CDEFMT_USE_DYNAMIC_LOG_BUFFER) && CDEFMT_USE_DYNAMIC_LOG_BUFFER */

  free(dynamic_struct);

#if defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER
  pthread_mutex_destroy(&cdefmt_global_buffer_lock);
#endif /* defined (CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER */

  return 0;
}

void cdefmt_log(const void* log, size_t size, enum cdefmt_level level) {
  struct stat stat;

  fstat(1, &stat);

  if (S_ISFIFO(stat.st_mode)) {
    static_assert(sizeof(uint64_t) >= sizeof(size_t));
    const uint64_t size_u64 = size;

    // Write raw binary data as length value pairs.
    fwrite(&size_u64, sizeof(size_u64), 1, stdout);
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
