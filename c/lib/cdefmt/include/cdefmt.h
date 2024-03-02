#ifndef CDEFMT_H
#define CDEFMT_H

#include <stddef.h>

/* Inner mechanisms */

#define __CDEFMT_LEVEL_ERR 0
#define __CDEFMT_LEVEL_WRN 1
#define __CDEFMT_LEVEL_INF 2
#define __CDEFMT_LEVEL_DBG 3
#define __CDEFMT_LEVEL_VRB 4

/* User API */

enum cdefmt_level {
  CDEFMT_LEVEL_ERR = __CDEFMT_LEVEL_ERR,
  CDEFMT_LEVEL_WRN = __CDEFMT_LEVEL_WRN,
  CDEFMT_LEVEL_INF = __CDEFMT_LEVEL_INF,
  CDEFMT_LEVEL_DBG = __CDEFMT_LEVEL_DBG,
  CDEFMT_LEVEL_VRB = __CDEFMT_LEVEL_VRB,
};

#define CDEFMT_ERROR(...)   __CDEFMT_LOG(__CDEFMT_LEVEL_ERR, __VA_ARGS__)
#define CDEFMT_WARNING(...) __CDEFMT_LOG(__CDEFMT_LEVEL_WRN, __VA_ARGS__)
#define CDEFMT_INFO(...)    __CDEFMT_LOG(__CDEFMT_LEVEL_INF, __VA_ARGS__)
#define CDEFMT_DEBUG(...)   __CDEFMT_LOG(__CDEFMT_LEVEL_DBG, __VA_ARGS__)
#define CDEFMT_VERBOSE(...) __CDEFMT_LOG(__CDEFMT_LEVEL_VRB, __VA_ARGS__)

/* Implement me */
void cdefmt_log(const void* log, size_t size, enum cdefmt_level level);

/* Inner mechanisms */

/* Returns the 64th argument */
#define CDEFMT_ARG_N(_1, _2, _3, _4, _5, _6, _7, _8, _9, _10, _11, _12, _13, _14, _15, _16, _17, \
                     _18, _19, _20, _21, _22, _23, _24, _25, _26, _27, _28, _29, _30, _31, _32,  \
                     _33, _34, _35, _36, _37, _38, _39, _40, _41, _42, _43, _44, _45, _46, _47,  \
                     _48, _49, _50, _51, _52, _53, _54, _55, _56, _57, _58, _59, _60, _61, _62,  \
                     _63, N, ...)                                                                \
  N

/* Returns a sequence of 64 numbers, counting down from 63 to 0 */
#define CDEFMT_RSEQ_N()                                                                           \
  63, 62, 61, 60, 59, 58, 57, 56, 55, 54, 53, 52, 51, 50, 49, 48, 47, 46, 45, 44, 43, 42, 41, 40, \
      39, 38, 37, 36, 35, 34, 33, 32, 31, 30, 29, 28, 27, 26, 25, 24, 23, 22, 21, 20, 19, 18, 17, \
      16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1, 0

/* Returns amount of arguments */
#define CDEFMT_NARG(...)   __CDEFMT_NARG(__VA_ARGS__, CDEFMT_RSEQ_N())
#define __CDEFMT_NARG(...) CDEFMT_ARG_N(__VA_ARGS__)

/* Concatenates arguments into a single literal */
#define CDEFMT_CONCAT(a_, b_) a_##b_

#define CDEFMT_STRINGIFY(a_)            __CDEFMT_STRINGIFY_INDIRECT(a_)
#define __CDEFMT_STRINGIFY_INDIRECT(a_) #a_

/* Allows overloading functions depending on amount of arguments, limited to at
 * least one argument */
#define CDEFMT_OVERLOAD(func_name_, ...) \
  __CDEFMT_OVERLOAD_INDIRECT(func_name_, CDEFMT_NARG(__VA_ARGS__))
#define __CDEFMT_OVERLOAD_INDIRECT(func_name_, nargs_) CDEFMT_CONCAT(func_name_, nargs_)

#define CDEFMT_LOG_STRING(counter_) CDEFMT_CONCAT(cdefmt_log_string, counter_)
#define CDEFMT_LOG_ARGS_T(counter_) CDEFMT_CONCAT(cdefmt_log_args_t, counter_)
#define CDEFMT_LOG_ARGS(counter_)   CDEFMT_CONCAT(cdefmt_log_args, counter_)

#define CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_) \
  "{\"counter\":"CDEFMT_STRINGIFY(counter_)"," \
  "\"level\":"CDEFMT_STRINGIFY(level_)"," \
  "\"file\":\""file_"\"," \
  "\"line\":"CDEFMT_STRINGIFY(line_)"," \
  "\"message\":\""message_"\"}"

/* Expands into a call to the correct log macro, the first argument of the
 * __VA_ARGS__ is the message (as it's required), this is done to overcome the
 * limitation of having at least one argument */
#define __CDEFMT_LOG(level_, ...)            \
  CDEFMT_OVERLOAD(__CDEFMT_LOG, __VA_ARGS__) \
  (__COUNTER__, level_, __FILE__, __LINE__, __VA_ARGS__)

/* Log with 0 arguments */
#define __CDEFMT_LOG1(counter_, level_, file_, line_, message_)                           \
  do {                                                                                    \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] = \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                  \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                          \
      const char* log_id;                                                                 \
    };                                                                                    \
                                                                                          \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                      \
        .log_id = CDEFMT_LOG_STRING(counter_),                                            \
    };                                                                                    \
                                                                                          \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);    \
  } while (0)

/* Log with 1 argument */
#define __CDEFMT_LOG2(counter_, level_, file_, line_, message_, arg1_)                    \
  do {                                                                                    \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] = \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                  \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                          \
      const char* log_id;                                                                 \
      const __typeof__(arg1_) arg1;                                                       \
    };                                                                                    \
                                                                                          \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                      \
        .log_id = CDEFMT_LOG_STRING(counter_),                                            \
        .arg1 = arg1_,                                                                    \
    };                                                                                    \
                                                                                          \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);    \
  } while (0)

/* Log with 2 arguments */
#define __CDEFMT_LOG3(counter_, level_, file_, line_, message_, arg1_, arg2_)             \
  do {                                                                                    \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] = \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                  \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                          \
      const char* log_id;                                                                 \
      const __typeof__(arg1_) arg1;                                                       \
      const __typeof__(arg2_) arg2;                                                       \
    };                                                                                    \
                                                                                          \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                      \
        .log_id = CDEFMT_LOG_STRING(counter_),                                            \
        .arg1 = arg1_,                                                                    \
        .arg2 = arg2_,                                                                    \
    };                                                                                    \
                                                                                          \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);    \
  } while (0)

/* Log with 3 arguments */
#define __CDEFMT_LOG4(counter_, level_, file_, line_, message_, arg1_, arg2_, arg3_)      \
  do {                                                                                    \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] = \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                  \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                          \
      const char* log_id;                                                                 \
      const __typeof__(arg1_) arg1;                                                       \
      const __typeof__(arg2_) arg2;                                                       \
      const __typeof__(arg3_) arg3;                                                       \
    };                                                                                    \
                                                                                          \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                      \
        .log_id = CDEFMT_LOG_STRING(counter_),                                            \
        .arg1 = arg1_,                                                                    \
        .arg2 = arg2_,                                                                    \
        .arg3 = arg3_,                                                                    \
    };                                                                                    \
                                                                                          \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);    \
  } while (0)

/* Log with 4 arguments */
#define __CDEFMT_LOG5(counter_, level_, file_, line_, message_, arg1_, arg2_, arg3_, arg4_) \
  do {                                                                                      \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] =   \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                    \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                            \
      const char* log_id;                                                                   \
      const __typeof__(arg1_) arg1;                                                         \
      const __typeof__(arg2_) arg2;                                                         \
      const __typeof__(arg3_) arg3;                                                         \
      const __typeof__(arg4_) arg4;                                                         \
    };                                                                                      \
                                                                                            \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                        \
        .log_id = CDEFMT_LOG_STRING(counter_),                                              \
        .arg1 = arg1_,                                                                      \
        .arg2 = arg2_,                                                                      \
        .arg3 = arg3_,                                                                      \
        .arg4 = arg4_,                                                                      \
    };                                                                                      \
                                                                                            \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);      \
  } while (0)

/* Log with 5 arguments */
#define __CDEFMT_LOG6(counter_, level_, file_, line_, message_, arg1_, arg2_, arg3_, arg4_, arg5_) \
  do {                                                                                             \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] =          \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                           \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                                   \
      const char* log_id;                                                                          \
      const __typeof__(arg1_) arg1;                                                                \
      const __typeof__(arg2_) arg2;                                                                \
      const __typeof__(arg3_) arg3;                                                                \
      const __typeof__(arg4_) arg4;                                                                \
      const __typeof__(arg5_) arg5;                                                                \
    };                                                                                             \
                                                                                                   \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                               \
        .log_id = CDEFMT_LOG_STRING(counter_),                                                     \
        .arg1 = arg1_,                                                                             \
        .arg2 = arg2_,                                                                             \
        .arg3 = arg3_,                                                                             \
        .arg4 = arg4_,                                                                             \
        .arg5 = arg5_,                                                                             \
    };                                                                                             \
                                                                                                   \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);             \
  } while (0)

/* Log with 6 arguments */
#define __CDEFMT_LOG7(counter_, level_, file_, line_, message_, arg1_, arg2_, arg3_, arg4_, arg5_, \
                      arg6_)                                                                       \
  do {                                                                                             \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] =          \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                           \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                                   \
      const char* log_id;                                                                          \
      const __typeof__(arg1_) arg1;                                                                \
      const __typeof__(arg2_) arg2;                                                                \
      const __typeof__(arg3_) arg3;                                                                \
      const __typeof__(arg4_) arg4;                                                                \
      const __typeof__(arg5_) arg5;                                                                \
      const __typeof__(arg6_) arg6;                                                                \
    };                                                                                             \
                                                                                                   \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                               \
        .log_id = CDEFMT_LOG_STRING(counter_),                                                     \
        .arg1 = arg1_,                                                                             \
        .arg2 = arg2_,                                                                             \
        .arg3 = arg3_,                                                                             \
        .arg4 = arg4_,                                                                             \
        .arg5 = arg5_,                                                                             \
        .arg6 = arg6_,                                                                             \
    };                                                                                             \
                                                                                                   \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);             \
  } while (0)

/* Log with 7 arguments */
#define __CDEFMT_LOG8(counter_, level_, file_, line_, message_, arg1_, arg2_, arg3_, arg4_, arg5_, \
                      arg6_, arg7_)                                                                \
  do {                                                                                             \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] =          \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                           \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                                   \
      const char* log_id;                                                                          \
      const __typeof__(arg1_) arg1;                                                                \
      const __typeof__(arg2_) arg2;                                                                \
      const __typeof__(arg3_) arg3;                                                                \
      const __typeof__(arg4_) arg4;                                                                \
      const __typeof__(arg5_) arg5;                                                                \
      const __typeof__(arg6_) arg6;                                                                \
      const __typeof__(arg7_) arg7;                                                                \
    };                                                                                             \
                                                                                                   \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                               \
        .log_id = CDEFMT_LOG_STRING(counter_),                                                     \
        .arg1 = arg1_,                                                                             \
        .arg2 = arg2_,                                                                             \
        .arg3 = arg3_,                                                                             \
        .arg4 = arg4_,                                                                             \
        .arg5 = arg5_,                                                                             \
        .arg6 = arg6_,                                                                             \
        .arg7 = arg7_,                                                                             \
    };                                                                                             \
                                                                                                   \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);             \
  } while (0)

/* Log with 8 arguments */
#define __CDEFMT_LOG9(counter_, level_, file_, line_, message_, arg1_, arg2_, arg3_, arg4_, arg5_, \
                      arg6_, arg7_, arg8_)                                                         \
  do {                                                                                             \
    const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] =          \
        CDEFMT_FORMAT_MESSAGE(counter_, level_, file_, line_, message_);                           \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                                   \
      const char* log_id;                                                                          \
      const __typeof__(arg1_) arg1;                                                                \
      const __typeof__(arg2_) arg2;                                                                \
      const __typeof__(arg3_) arg3;                                                                \
      const __typeof__(arg4_) arg4;                                                                \
      const __typeof__(arg5_) arg5;                                                                \
      const __typeof__(arg6_) arg6;                                                                \
      const __typeof__(arg7_) arg7;                                                                \
      const __typeof__(arg8_) arg8;                                                                \
    };                                                                                             \
                                                                                                   \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                               \
        .log_id = CDEFMT_LOG_STRING(counter_),                                                     \
        .arg1 = arg1_,                                                                             \
        .arg2 = arg2_,                                                                             \
        .arg3 = arg3_,                                                                             \
        .arg4 = arg4_,                                                                             \
        .arg5 = arg5_,                                                                             \
        .arg6 = arg6_,                                                                             \
        .arg7 = arg7_,                                                                             \
        .arg8 = arg8_,                                                                             \
    };                                                                                             \
                                                                                                   \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);             \
  } while (0)

#endif /* CDEFMT_H */