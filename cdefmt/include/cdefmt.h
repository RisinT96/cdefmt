#ifndef CDEFMT_H
#define CDEFMT_H

#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include <boost/preprocessor.hpp>

/* Inner mechanisms */

#define __CDEFMT_LEVEL_ERR 0
#define __CDEFMT_LEVEL_WRN 1
#define __CDEFMT_LEVEL_INF 2
#define __CDEFMT_LEVEL_DBG 3
#define __CDEFMT_LEVEL_VRB 4

/* User API */

static inline int cdefmt_init();
#define CDEFMT_GENERATE_INIT() __CDEFMT_GENERATE_INIT()

enum cdefmt_level {
  CDEFMT_LEVEL_ERR = __CDEFMT_LEVEL_ERR,
  CDEFMT_LEVEL_WRN = __CDEFMT_LEVEL_WRN,
  CDEFMT_LEVEL_INF = __CDEFMT_LEVEL_INF,
  CDEFMT_LEVEL_DBG = __CDEFMT_LEVEL_DBG,
  CDEFMT_LEVEL_VRB = __CDEFMT_LEVEL_VRB,
};

#define CDEFMT_ERROR(message_, ...)   _CDEFMT_LOG(__CDEFMT_LEVEL_ERR, message_, ##__VA_ARGS__)
#define CDEFMT_WARNING(message_, ...) _CDEFMT_LOG(__CDEFMT_LEVEL_WRN, message_, ##__VA_ARGS__)
#define CDEFMT_INFO(message_, ...)    _CDEFMT_LOG(__CDEFMT_LEVEL_INF, message_, ##__VA_ARGS__)
#define CDEFMT_DEBUG(message_, ...)   _CDEFMT_LOG(__CDEFMT_LEVEL_DBG, message_, ##__VA_ARGS__)
#define CDEFMT_VERBOSE(message_, ...) _CDEFMT_LOG(__CDEFMT_LEVEL_VRB, message_, ##__VA_ARGS__)

/**
 * cdefmt_log() - Log sink for all logs printed by cdefmt.
 *                Has to be implemented by the library's user.
 * @log:   pointer to the log object.
 * @size:  the size of the log object.
 * @level: the log's level.
 *
 * This function can filter the logs at runtime based on the `level`.
 * The implementation has to write the `log` into the log backends used by the project.
 */
void cdefmt_log(const void* log, size_t size, enum cdefmt_level level);

/* Inner mechanisms */

#define CDEFMT_SCHEMA_VERSION    1
#define CDEFMT_GNU_BUILD_ID_SIZE 20

#define __CDEFMT_GENERATE_METADATA_ARG_NAME(r_, _, i_, elem_) \
  BOOST_PP_IF(i_, ",", ) "\"" BOOST_PP_STRINGIZE(elem_)"\""
#define CDEFMT_GENERATE_METADATA_ARG_NAMES(args_seq_) \
  BOOST_PP_SEQ_FOR_EACH_I(__CDEFMT_GENERATE_METADATA_ARG_NAME, _, args_seq_)

#define CDEFMT_FORMAT_METADATA(counter_, level_, file_, line_, message_, args_seq_) \
  "{"                                                                               \
      "\"version\":"BOOST_PP_STRINGIZE(CDEFMT_SCHEMA_VERSION)","                    \
      "\"counter\":"BOOST_PP_STRINGIZE(counter_)","                                 \
      "\"level\":"BOOST_PP_STRINGIZE(level_)","                                     \
      "\"file\":\""file_"\","                                                       \
      "\"line\":"BOOST_PP_STRINGIZE(line_)","                                       \
      "\"message\":\""message_"\","                                                 \
      "\"names\": ["                                                                \
          CDEFMT_GENERATE_METADATA_ARG_NAMES(args_seq_)                             \
      "]"                                                                           \
  "}"

#define CDEFMT_LOG_STRING(counter_) BOOST_PP_CAT(cdefmt_log_string, counter_)
#define CDEFMT_LOG_ARGS_T(counter_) BOOST_PP_CAT(cdefmt_log_args_t, counter_)
#define CDEFMT_LOG_ARGS(counter_)   BOOST_PP_CAT(cdefmt_log_args, counter_)

#define CDEFMT_GENERATE_METADATA_STRING(counter_, level_, file_, line_, message_, args_seq_) \
  const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] =      \
      CDEFMT_FORMAT_METADATA(counter_, level_, file_, line_, message_, args_seq_)

#define __CDEFMT_GENERATE_LOG_ARG(r_, _, i_, elem_) __typeof__(elem_) arg##i_;
#define CDEFMT_GENERATE_LOG_ARGS(args_seq_) \
  BOOST_PP_SEQ_FOR_EACH_I(__CDEFMT_GENERATE_LOG_ARG, _, args_seq_)

#define CDEFMT_ASSIGN(to_, from_)          \
  do {                                     \
    memcpy((&to_), &(from_), sizeof(to_)); \
  } while (0)

#define __CDEFMT_ASSIGN_LOG_ARG(r_, counter_, i_, elem_) \
  CDEFMT_ASSIGN(CDEFMT_LOG_ARGS(counter_).arg##i_, elem_);
#define CDEFMT_ASSIGN_LOG_ARGS(counter_, args_seq_) \
  BOOST_PP_SEQ_FOR_EACH_I(__CDEFMT_ASSIGN_LOG_ARG, counter_, args_seq_)

#define __CDEFMT_LOG(counter_, level_, file_, line_, message_, args_seq_)                 \
  do {                                                                                    \
    CDEFMT_GENERATE_METADATA_STRING(counter_, level_, file_, line_, message_, args_seq_); \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                          \
      const char* log_id;                                                                 \
      CDEFMT_GENERATE_LOG_ARGS(args_seq_)                                                 \
    };                                                                                    \
                                                                                          \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                      \
        .log_id = CDEFMT_LOG_STRING(counter_),                                            \
    };                                                                                    \
    CDEFMT_ASSIGN_LOG_ARGS(counter_, args_seq_)                                           \
                                                                                          \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), level_);    \
  } while (0)

/* Need a level of indirection mainly to expand `__COUNTER__`, `__FILE__` and `__LINE__`
 * Additionally, for easier manipulation we're turning all the __VA_ARGS__ into a SEQ.
 * The SEQ generation is a bit tricky and depends on the GNU ## extension:
 * - If __VA_ARGS__ is empty, the `,` will be removed and we'll get a SEQ with 1 empty element: `()`
 * - If __VA_ARGS__ is not empty, we'll get a SEQ with 1 empty element followed by the actual
 *   arguments: `()(arg1)(arg2)...`
 * We then pop the first element and end up with a SEQ that only contains the arguments.
 */
#define _CDEFMT_LOG(level_, message_, ...)                        \
  __CDEFMT_LOG(__COUNTER__, level_, __FILE__, __LINE__, message_, \
               BOOST_PP_SEQ_POP_FRONT(BOOST_PP_VARIADIC_TO_SEQ(, ##__VA_ARGS__)))

struct cdefmt_build_id {
  uint32_t name_size;
  uint32_t data_size;
  uint32_t type;
  uint8_t data[];
};

#ifndef NT_GNU_BUILD_ID
#define NT_GNU_BUILD_ID 3
#endif

#define __CDEFMT_INIT(counter_)                                                                    \
  do {                                                                                             \
    const static __attribute__((section(".cdefmt.init"))) char CDEFMT_LOG_STRING(counter_)[] =     \
        CDEFMT_FORMAT_METADATA(counter_, __CDEFMT_LEVEL_ERR, __FILE__, 0, "cdefmt init: {}",);      \
                                                                                                   \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                                   \
      const char* log_id;                                                                          \
      unsigned char build_id[CDEFMT_GNU_BUILD_ID_SIZE];                                            \
    };                                                                                             \
                                                                                                   \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                               \
        .log_id = CDEFMT_LOG_STRING(counter_),                                                     \
    };                                                                                             \
    CDEFMT_ASSIGN(CDEFMT_LOG_ARGS(counter_).build_id,                                              \
                  __cdefmt_build_id.data[__cdefmt_build_id.name_size]);                            \
                                                                                                   \
    cdefmt_log(&CDEFMT_LOG_ARGS(counter_), sizeof(CDEFMT_LOG_ARGS(counter_)), __CDEFMT_LEVEL_ERR); \
  } while (0)

#define __CDEFMT_GENERATE_INIT()                                   \
  static inline int cdefmt_init() {                                \
    extern const struct cdefmt_build_id __cdefmt_build_id;         \
    if (__cdefmt_build_id.type != NT_GNU_BUILD_ID) {               \
      return -1;                                                   \
    }                                                              \
                                                                   \
    if (__cdefmt_build_id.data_size != CDEFMT_GNU_BUILD_ID_SIZE) { \
      return -2;                                                   \
    };                                                             \
                                                                   \
    __CDEFMT_INIT(__COUNTER__);                                    \
                                                                   \
    return 0;                                                      \
  }

#endif /* CDEFMT_H */
