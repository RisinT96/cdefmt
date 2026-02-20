#include <cdefmt/include/cdefmt.h>
#include <locale.h>
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

CDEFMT_GENERATE_INIT()

static FILE* output;

// Helper to print a double with comma thousands separators (for integer part)
void print_double_commas(double value, int precision) {
  long long int_part = (long long)value;
  double frac_part = value - int_part;
  char buf[64];
  char out[128];
  int len = 0, outlen = 0, i, comma_count = 0;
  snprintf(buf, sizeof(buf), "%lld", int_part);
  len = strlen(buf);
  // Print integer part with commas
  for (i = 0; i < len; ++i) {
    out[outlen++] = buf[i];
    if (((len - i - 1) % 3 == 0) && (i != len - 1)) {
      out[outlen++] = ',';
    }
  }
  out[outlen] = '\0';
  // Print fractional part
  if (precision > 0) {
    char fracbuf[32];
    snprintf(fracbuf, sizeof(fracbuf), "%.*f", precision, frac_part);
    // fracbuf starts with "0."
    strcat(out, &fracbuf[1]);
  }
  printf("%s", out);
}

int main(int argc, char* cargv[]) {
#include <locale.h>
  setlocale(LC_NUMERIC, "");
#if defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER
  pthread_mutex_init(&cdefmt_global_buffer_lock, NULL);
#endif /* defined (CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER */

  // Open output file for writing
  output = fopen("output.log", "wb");
  if (!output) {
    perror("Failed to open output file");
    return 1;
  }

  if (cdefmt_init()) {
    fclose(output);
    return 1;
  }

  struct timespec start;
  if (clock_gettime(CLOCK_MONOTONIC, &start) != 0) {
    perror("Failed to get current time");
    fclose(output);
    return 1;
  }

  const size_t iterations = 10000000;
  for (size_t i = 0; i < iterations; i++) {
    CDEFMT_INFO("Hello, message number {}", i);
  }

  struct timespec end;
  if (clock_gettime(CLOCK_MONOTONIC, &end) != 0) {
    perror("Failed to get current time");
    fclose(output);
    return 1;
  }

  // Calculate elapsed time in seconds
  double elapsed = (end.tv_sec - start.tv_sec) + (end.tv_nsec - start.tv_nsec) / 1e9;
  double iter_per_sec = iterations / elapsed;

  setlocale(LC_NUMERIC, "");
  printf("Elapsed time: %.6f seconds\n", elapsed);
  printf("Iterations/sec: ");
  print_double_commas(iter_per_sec, 2);
  printf("\n");

#if defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER
  pthread_mutex_destroy(&cdefmt_global_buffer_lock);
#endif /* defined (CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER */

  // Close output file before exiting
  fclose(output);
  return 0;
}

void cdefmt_log(const void* log, size_t size, enum cdefmt_level level) {
  static_assert(sizeof(uint64_t) >= sizeof(size_t));
  const uint64_t size_u64 = size;

  // Write raw binary data as length value pairs.
  fwrite(&size_u64, sizeof(size_u64), 1, output);
  fwrite(log, size, 1, output);
  return;
}
