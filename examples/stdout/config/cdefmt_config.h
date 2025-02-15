#ifndef CDEFMT_CONFIG_H
#define CDEFMT_CONFIG_H

/* ======================================== Stack Buffer ======================================== */

#define CDEFMT_USE_STACK_LOG_BUFFER 0

#if defined(CDEFMT_USE_STACK_LOG_BUFFER) && CDEFMT_USE_STACK_LOG_BUFFER
#define CDEFMT_STACK_LOG_BUFFER_DYNAMIC_SIZE_MAX 128
#endif /* defined (CDEFMT_USE_STACK_LOG_BUFFER) && CDEFMT_USE_STACK_LOG_BUFFER */

/* ======================================== Static Buffer ======================================= */

/* Use a global static log buffer, otherwise a buffer will be created on the stack on each
 * invocation of CDEFMT_LOG.
 * If enabled and running in a multi-threaded environment, the following functions must be
 * implemented by the user, as concurrent accesses to the buffer will cause corruption:
 * - CDEFMT_STATIC_LOG_BUFFER_LOCK()
 * - CDEFMT_STATIC_LOG_BUFFER_UNLOCK()
 * As well as the following defines:
 * - CDEFMT_STATIC_LOG_BUFFER      - accessor to the static log buffer.
 * - CDEFMT_STATIC_LOG_BUFFER_SIZE - the static log buffer's size (in bytes).
 */
#define CDEFMT_USE_STATIC_LOG_BUFFER 0

#if defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER
#include <pthread.h>
#include <stdint.h>

#define CDEFMT_STATIC_LOG_BUFFER_LOCK()   pthread_mutex_lock(&cdefmt_global_buffer_lock)
#define CDEFMT_STATIC_LOG_BUFFER_UNLOCK() pthread_mutex_unlock(&cdefmt_global_buffer_lock)
#define CDEFMT_STATIC_LOG_BUFFER          cdefmt_global_buffer
#define CDEFMT_STATIC_LOG_BUFFER_SIZE     512

extern uint8_t cdefmt_global_buffer[CDEFMT_STATIC_LOG_BUFFER_SIZE];
extern pthread_mutex_t cdefmt_global_buffer_lock;
#endif /* defined (CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER */

/* ======================================= Dynamic Buffer ======================================= */

/* Use a dynamically allocated log buffer, otherwise a buffer will be created on the stack on each
 * invocation of CDEFMT_LOG.
 * If enabled, a buffer will be allocated on the heap for each log, the following functions must be
 * implemented by the user:
 * - CDEFMT_DYNAMIC_LOG_BUFFER_ALLOC(size_)
 * - CDEFMT_DYNAMIC_LOG_BUFFER_FREE()
 */
#define CDEFMT_USE_DYNAMIC_LOG_BUFFER 1

#if defined(CDEFMT_USE_DYNAMIC_LOG_BUFFER) && CDEFMT_USE_DYNAMIC_LOG_BUFFER
#include <stdlib.h>

#define CDEFMT_DYNAMIC_LOG_BUFFER_ALLOC(size_)  calloc(1, size_)
#define CDEFMT_DYNAMIC_LOG_BUFFER_FREE(buffer_) free(buffer_)
#endif /* defined (CDEFMT_USE_DYNAMIC_LOG_BUFFER) && CDEFMT_USE_DYNAMIC_LOG_BUFFER */

#endif /* CDEFMT_CONFIG_H */
