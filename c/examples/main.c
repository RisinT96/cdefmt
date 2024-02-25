#include <stdint.h>
#include <stdio.h>
#include <sys/stat.h>

#include "cdefmt/include/cdefmt.h"

int main(int argc, char* cargv[]) {
  CDEFMT_ERROR("hello!");
  CDEFMT_ERROR("WHAT {}", 123);
  CDEFMT_ERROR("WHAT {0} {1} {2} {3} {4} {5} {6} {7}", 1, 2, 3, 4, 5, 6, 7, 8);
  CDEFMT_WARNING("WHAT {0} {1} {2} {3} {4} {5} {6} {7}", 1, 2, 3, 4, 5, 6, 7, 8);
  CDEFMT_INFO("WHAT {0} {1} {2} {3} {4} {5} {6} {7}", 1, 2, 3, 4, 5, 6, 7, 8);
  CDEFMT_DEBUG("WHAT {0} {1} {2} {3} {4} {5} {6} {7}", 1, 2, 3, 4, 5, 6, 7, 8);
  CDEFMT_VERBOSE("WHAT {0} {1} {2} {3} {4} {5} {6} {7}", 1, 2, 3, 4, 5, 6, 7, 8);
}

void cdefmt_log(const void* log, size_t size, enum cdefmt_level level) {
  struct stat stat;

  fstat(1, &stat);

  if (S_ISFIFO(stat.st_mode)) {
    // For piping, we output the log IDs.
    printf("%lu\n", ((const uintptr_t*)log)[0]);
    fprintf(stderr, "shit: %lu\n", ((const uintptr_t*)log)[0]);
  } else {
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
}
