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

int main(int argc, char* cargv[]) {
  uint32_t a = 5;
  struct some_struct b = {
      .a = 123,
      .b = 543,
      .c = 9123,
  };
  bool c = false;

  CDEFMT_ERROR("WHAT?! {}", a);
  CDEFMT_WARNING("WHAT?! {}", 123);
  CDEFMT_INFO("WHAT?! {}", true);
  CDEFMT_DEBUG("WHAT?! {}", c, 123);
  CDEFMT_VERBOSE("I love spam! {}", b);

  return 0;
}

void cdefmt_log(const void* log, size_t size, enum cdefmt_level level) {
  struct stat stat;

  fstat(1, &stat);

  if (S_ISFIFO(stat.st_mode)) {
    // For piping, we output the log IDs.
    printf("%lu\n", ((const uintptr_t*)log)[0]);
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
