#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <sys/stat.h>

#include "cdefmt/include/cdefmt.h"

struct some_struct {
  uint32_t a;
  uint16_t b;
  uint64_t c;
};

enum some_enum {
  SOME_ENUM_FIRST,
  SOME_ENUM_WHAAAAT = 54,
  SOME_ENUM_LOLZ,
  SOME_ENUM_LAST = UINT64_MAX,
};

enum some_signed_enum {
  SOME_SIGNED_ENUM_1 = 0,
  SOME_SIGNED_ENUM_2 = INT32_MAX,
  SOME_SIGNED_ENUM_3 = INT32_MIN,
};

int main(int argc, char* cargv[]) {
  uint32_t a = 5;

  struct some_struct b = {
      .a = 123,
      .b = 543,
      .c = 9123,
  };

  bool c = false;

  enum some_enum e = SOME_ENUM_LOLZ;
  enum some_signed_enum e2 = SOME_SIGNED_ENUM_2;

  CDEFMT_ERROR("What just happened?! {} {}", a, e2);
  CDEFMT_WARNING("This wasn't supposed to happen... {}", 123);
  CDEFMT_INFO("Just letting you know: {}", true);
  CDEFMT_DEBUG("Oh so you like debugging: {} {}", c, 123);
  CDEFMT_VERBOSE("I love spam! {} {} {} {}", a, b, c, e);

  return 0;
}

void cdefmt_log(const void* log, size_t size, enum cdefmt_level level) {
  struct stat stat;

  fstat(1, &stat);

  if (S_ISFIFO(stat.st_mode)) {
    // When piping, write the data as hex
    for (size_t i = 0; i < size; i++) {
      printf("%02x", ((const uint8_t*)log)[i]);

      if (i + 1 < size) {
        printf(";");
      }
    }

    printf("\n");
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
