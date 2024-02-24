#include <cdefmt/include/cdefmt.h>
#include <stdio.h>

int main(int argc, char* cargv[]) {
  CDEFMT_INFO("HELLO\n");
  CDEFMT_INFO("HELLO 2\n");
  CDEFMT_INFO("HELLO 3\n");
  CDEFMT_INFO("HELLO 3\n");
  CDEFMT_VERBOSE("HELLO\n");
  CDEFMT_DEBUG("HELLO 2\n");
  CDEFMT_INFO("HELLO 2\n");
  CDEFMT_WARNING("HELLO 2\n");
  CDEFMT_ERROR("HELLO 2\n");
  CDEFMT_ERROR("WHAT %u %u %u\n", 123);
  CDEFMT_ERROR("WHAT %u %u %u %u %u %u %u %u %u %u\n", 1, 2, 3, 4, 5, 6, 7, 8);
}

void cdefmt_log(const void* log, size_t size, int level) {
  printf("[%u] %p:%zu, log_id: %p\n", level, log, size, ((const char**)log)[0]);
}
