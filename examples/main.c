#include <stdint.h>
#include <stdio.h>

#include "cdefmt/include/cdefmt.h"

int main(int argc, char* cargv[]) {
  CDEFMT_ERROR("WHAT {}", 123);
  CDEFMT_ERROR("WHAT {0} {1} {2} {3} {4} {5} {6} {7}", 1, 2, 3, 4, 5, 6, 7, 8);
}

void cdefmt_log(const void* log, size_t size, enum cdefmt_level level) {
  printf("level: %u, id: %#010lx, size: %-3zu data: [", level,
         ((const uintptr_t*)log)[0], size);

  for (size_t i = 0; i < size; i++) {
    printf("%02x", ((const uint8_t*)log)[i]);

    if (i + 1 < size) {
      printf(", ");
    }
  }

  printf("]\n");
}
