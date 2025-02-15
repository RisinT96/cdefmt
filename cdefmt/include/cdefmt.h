#ifndef CDEFMT_H
#define CDEFMT_H

#include <config/cdefmt_config.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#include "boost/preprocessor/seq/for_each_i.hpp"
#include "boost/preprocessor/seq/pop_front.hpp"
#include "boost/preprocessor/stringize.hpp"
#include "boost/preprocessor/variadic/to_seq.hpp"
#include "boost/vmd/is_tuple.hpp"

/* Inner mechanisms */

#define __CDEFMT_LEVEL_ERR 0
#define __CDEFMT_LEVEL_WRN 1
#define __CDEFMT_LEVEL_INF 2
#define __CDEFMT_LEVEL_DBG 3
#define __CDEFMT_LEVEL_VRB 4

/* ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ User APIs ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ */

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

#define CDEFMT_DYNAMIC_ARRAY(array_, length_) \
  __CDEFMT_PARAMETER(DYNAMIC_ARRAY, __CDEFMT_DYNAMIC_ARRAY(array_, length_))
#define CDEFMT_DYNAMIC_STRING(string_) CDEFMT_DYNAMIC_ARRAY(string_, strlen(string_))
#define CDEFMT_DYNAMIC_STRING_N(string_, max_len_) \
  CDEFMT_DYNAMIC_ARRAY(string_, strnlen(string_, max_len_))

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

/* ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ Config Checks ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ */

#if (defined(CDEFMT_USE_STACK_LOG_BUFFER) && CDEFMT_USE_STACK_LOG_BUFFER)

/* Stack log buffer */

#define CDEFMT_DYNAMIC_SIZE_MAX_T         CDEFMT_STACK_LOG_BUFFER_DYNAMIC_SIZE_MAX
#define CDEFMT_DYNAMIC_SIZE_MAX(counter_) CDEFMT_STACK_LOG_BUFFER_DYNAMIC_SIZE_MAX

#define CDEFMT_GET_LOG_BUFFER(counter_, size_) (&(struct CDEFMT_LOG_ARGS_T(counter_)){0})
#define CDEFMT_RELEASE_LOG_BUFFER(counter_)

#define CDEFMT_GET_DYNAMIC_LOG_BUFFER_SIZE(counter_) \
  (sizeof(*CDEFMT_LOG_ARGS(counter_)) - (CDEFMT_DYNAMIC_SIZE_MAX_T - cdefmt_dynamic_offset))

#elif (defined(CDEFMT_USE_STATIC_LOG_BUFFER) && CDEFMT_USE_STATIC_LOG_BUFFER)

/* Static log buffer */

#define CDEFMT_DYNAMIC_SIZE_MAX_T 0
#define CDEFMT_DYNAMIC_SIZE_MAX(counter_) \
  (CDEFMT_STATIC_LOG_BUFFER_SIZE - sizeof(struct CDEFMT_LOG_ARGS_T(counter_)))

#define CDEFMT_GET_LOG_BUFFER(counter_, size_)                     \
  ({                                                               \
    CDEFMT_STATIC_LOG_BUFFER_LOCK();                               \
    (struct CDEFMT_LOG_ARGS_T(counter_)*)CDEFMT_STATIC_LOG_BUFFER; \
  })
#define CDEFMT_RELEASE_LOG_BUFFER(counter_) CDEFMT_STATIC_LOG_BUFFER_UNLOCK()

#define CDEFMT_GET_DYNAMIC_LOG_BUFFER_SIZE(counter_) \
  (sizeof(*CDEFMT_LOG_ARGS(counter_)) + cdefmt_dynamic_size)

#elif (defined(CDEFMT_USE_DYNAMIC_LOG_BUFFER) && CDEFMT_USE_DYNAMIC_LOG_BUFFER)

/* Dynamic log buffer */

#define CDEFMT_DYNAMIC_SIZE_MAX_T         0
#define CDEFMT_DYNAMIC_SIZE_MAX(counter_) (cdefmt_dynamic_size)

#define CDEFMT_GET_LOG_BUFFER(counter_, size_) \
  ((struct CDEFMT_LOG_ARGS_T(counter_)*)CDEFMT_DYNAMIC_LOG_BUFFER_ALLOC(size_))
#define CDEFMT_RELEASE_LOG_BUFFER(counter_) \
  CDEFMT_DYNAMIC_LOG_BUFFER_FREE(CDEFMT_LOG_ARGS(counter_))

#define CDEFMT_GET_DYNAMIC_LOG_BUFFER_SIZE(counter_) \
  (sizeof(*CDEFMT_LOG_ARGS(counter_)) + cdefmt_dynamic_size)

#else

/* Use impossible include to stop compiler before it starts spouting a ton of useless errors */
#include "No log buffer type selected, please select one of the supported types!"

#endif

#if ((CDEFMT_USE_STACK_LOG_BUFFER + CDEFMT_USE_STATIC_LOG_BUFFER + \
      CDEFMT_USE_DYNAMIC_LOG_BUFFER) > 1)
/* Use impossible include to stop compiler before it starts spouting a ton of useless errors */
#include "Please select just one log buffer type!"
#endif

/* ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ Private APIs ≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡≡ */

#define CDEFMT_SCHEMA_VERSION    1
#define CDEFMT_GNU_BUILD_ID_SIZE 20

/* ======================================== Special Types ======================================= */

/* ---------------------------------- General Special Parameter --------------------------------- */

#define __CDEFMT_PARAMETER(type_, value_)      (type_, value_)
#define CDEFMT_PARAMETER_GET_TYPE(parameter_)  BOOST_PP_TUPLE_ELEM(0, parameter_)
#define CDEFMT_PARAMETER_GET_VALUE(parameter_) BOOST_PP_TUPLE_ELEM(1, parameter_)

/* Generate argument name for metadata string */
#define __CDEFMT_GENERATE_METADATA_ARG_NAME_PARAMETER(parameter_) \
  BOOST_PP_CAT(__CDEFMT_GENERATE_METADATA_ARG_NAME_,              \
               CDEFMT_PARAMETER_GET_TYPE(parameter_))(CDEFMT_PARAMETER_GET_VALUE(parameter_))

/* Generate field for log struct */
#define __CDEFMT_GENERATE_LOG_ARG_PARAMETER(counter_, i_, parameter_)              \
  BOOST_PP_CAT(__CDEFMT_GENERATE_LOG_ARG_, CDEFMT_PARAMETER_GET_TYPE(parameter_))( \
      counter_, i_, CDEFMT_PARAMETER_GET_VALUE(parameter_))

/* Assign log struct field */
#define __CDEFMT_ASSIGN_LOG_ARG_PARAMETER(counter_, i_, parameter_)              \
  BOOST_PP_CAT(__CDEFMT_ASSIGN_LOG_ARG_, CDEFMT_PARAMETER_GET_TYPE(parameter_))( \
      counter_, i_, CDEFMT_PARAMETER_GET_VALUE(parameter_))

/* Calculate dynamic size */
#define __CDEFMT_CALCULATE_DYNAMIC_SIZE_PARAMETER(counter_, i_, parameter_)              \
  BOOST_PP_CAT(__CDEFMT_CALCULATE_DYNAMIC_SIZE_, CDEFMT_PARAMETER_GET_TYPE(parameter_))( \
      counter_, i_, CDEFMT_PARAMETER_GET_VALUE(parameter_))

/* ---------------------------------------- Dynamic Array --------------------------------------- */

#define __CDEFMT_DYNAMIC_ARRAY(array_, length_)         (array_, length_)
#define CDEFMT_DYNAMIC_ARRAY_GET_ARRAY(dynamic_array_)  BOOST_PP_TUPLE_ELEM(0, dynamic_array_)
#define CDEFMT_DYNAMIC_ARRAY_GET_LENGTH(dynamic_array_) BOOST_PP_TUPLE_ELEM(1, dynamic_array_)

#define __CDEFMT_GENERATE_METADATA_ARG_NAME_DYNAMIC_ARRAY(dynamic_array_) \
  BOOST_PP_STRINGIZE(CDEFMT_DYNAMIC_ARRAY_GET_ARRAY(dynamic_array_))

/* We put length information into the field, and encode the type using a zero length array.
 * The actual data will be appended at the end of the arguments array, in the dynamic array */
#define __CDEFMT_GENERATE_LOG_ARG_DYNAMIC_ARRAY(counter_, i_, dynamic_array_) \
  struct __attribute__((packed)) {                                            \
    size_t size;                                                              \
    __typeof__(*CDEFMT_DYNAMIC_ARRAY_GET_ARRAY(dynamic_array_)) type[0];      \
  } dynamic_array##_##i_

/* Assign argument value and size information */
#define __CDEFMT_ASSIGN_LOG_ARG_DYNAMIC_ARRAY_INNER(counter_, i_, array_, length_)                \
  do {                                                                                            \
    size_t cdefmt_increment = 0;                                                                  \
    if (cdefmt_dynamic_offset < (CDEFMT_DYNAMIC_SIZE_MAX(counter_))) {                            \
      /* Minimum between array size, and how much space is left for dynamic data */               \
      cdefmt_increment = CDEFMT_MIN(sizeof(*(array_)) * (length_),                                \
                                    (CDEFMT_DYNAMIC_SIZE_MAX(counter_)) - cdefmt_dynamic_offset); \
      memcpy(CDEFMT_LOG_ARGS(counter_)->dynamic_data + cdefmt_dynamic_offset, (array_),           \
             cdefmt_increment);                                                                   \
      cdefmt_dynamic_offset += cdefmt_increment;                                                  \
    }                                                                                             \
    (CDEFMT_LOG_ARGS(counter_))->dynamic_array_##i_.size = cdefmt_increment;                      \
  } while (0)

#define __CDEFMT_ASSIGN_LOG_ARG_DYNAMIC_ARRAY(counter_, i_, dynamic_array_)                   \
  __CDEFMT_ASSIGN_LOG_ARG_DYNAMIC_ARRAY_INNER(counter_, i_,                                   \
                                              CDEFMT_DYNAMIC_ARRAY_GET_ARRAY(dynamic_array_), \
                                              CDEFMT_DYNAMIC_ARRAY_GET_LENGTH(dynamic_array_))

/* Add the parameter's length to the dynamic size */
#define __CDEFMT_CALCULATE_DYNAMIC_SIZE_DYNAMIC_ARRAY(counter_, i_, dynamic_array_)   \
  cdefmt_dynamic_size += (sizeof(*(CDEFMT_DYNAMIC_ARRAY_GET_ARRAY(dynamic_array_))) * \
                          (CDEFMT_DYNAMIC_ARRAY_GET_LENGTH(dynamic_array_)))

/* ======================================== Common Utils ======================================== */

/* Name of metadata string variable */
#define CDEFMT_LOG_STRING(counter_) BOOST_PP_CAT(cdefmt_log_string, counter_)

/* Name of log arguments type */
#define CDEFMT_LOG_ARGS_T(counter_) BOOST_PP_CAT(cdefmt_log_args_t, counter_)

/* Name of log arguments variable */
#define CDEFMT_LOG_ARGS(counter_) BOOST_PP_CAT(cdefmt_log_args, counter_)

/* Returns minumum between a and b */
#define CDEFMT_MIN(a, b)    \
  ({                        \
    __typeof__(a) _a = (a); \
    __typeof__(b) _b = (b); \
    _a <= _b ? _a : _b;     \
  })

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

#define __CDEFMT_LOG(counter_, level_, file_, line_, message_, args_seq_)                        \
  do {                                                                                           \
    /* Generate metadata string */                                                               \
    CDEFMT_GENERATE_METADATA_STRING(counter_, level_, file_, line_, message_, args_seq_);        \
                                                                                                 \
    /* Generate log type information */                                                          \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                                 \
      const char* log_id;                                                                        \
      CDEFMT_GENERATE_LOG_ARGS(counter_, args_seq_)                                              \
      uint8_t dynamic_data[CDEFMT_DYNAMIC_SIZE_MAX_T];                                           \
    };                                                                                           \
                                                                                                 \
    size_t cdefmt_dynamic_size = 0;                                                              \
    CDEFMT_CALCULATE_DYNAMIC_SIZE(counter_, args_seq_)                                           \
                                                                                                 \
    /* Create/get log buffer */                                                                  \
    struct CDEFMT_LOG_ARGS_T(counter_) * CDEFMT_LOG_ARGS(counter_) = CDEFMT_GET_LOG_BUFFER(      \
        counter_, sizeof(struct CDEFMT_LOG_ARGS_T(counter_)) + cdefmt_dynamic_size);             \
                                                                                                 \
    /* Assign log id */                                                                          \
    CDEFMT_LOG_ARGS(counter_)->log_id = CDEFMT_LOG_STRING(counter_);                             \
    size_t cdefmt_dynamic_offset = 0;                                                            \
                                                                                                 \
    /* Assign arguments */                                                                       \
    CDEFMT_ASSIGN_LOG_ARGS(counter_, args_seq_)                                                  \
                                                                                                 \
    /* Send log to sink */                                                                       \
    cdefmt_log(CDEFMT_LOG_ARGS(counter_), CDEFMT_GET_DYNAMIC_LOG_BUFFER_SIZE(counter_), level_); \
                                                                                                 \
    /* Release log buffer */                                                                     \
    CDEFMT_RELEASE_LOG_BUFFER(counter_);                                                         \
  } while (0)

/* ======================================= Metadata String ====================================== */

#define __CDEFMT_GENERATE_METADATA_ARG_NAME(r_, _, i_, elem_)                                     \
  /* Insert `,` before all elements that are not first */                                         \
  BOOST_PP_IF(i_, ",", )                                                                          \
  /* Insert stringified parameter surrounded by quotes */                                         \
  "\"" BOOST_PP_IF(BOOST_VMD_IS_TUPLE(elem_),                                                     \
                   __CDEFMT_GENERATE_METADATA_ARG_NAME_PARAMETER, /* Handle special parameters */ \
                   BOOST_PP_STRINGIZE)                            /* Handle regular parameters */ \
      (elem_) "\""

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

/* Generates entire metadata string variable */
#define CDEFMT_GENERATE_METADATA_STRING(counter_, level_, file_, line_, message_, args_seq_) \
  const static __attribute__((section(".cdefmt"))) char CDEFMT_LOG_STRING(counter_)[] =      \
      CDEFMT_FORMAT_METADATA(counter_, level_, file_, line_, message_, args_seq_)

/* ======================================== Log Argument ======================================== */

#define ___CDEFMT_GENERATE_LOG_ARG(counter_, i_, elem_) __typeof__(elem_) arg##i_
#define __CDEFMT_GENERATE_LOG_ARG(r_, counter_, i_, elem_)                         \
  BOOST_PP_IF(BOOST_VMD_IS_TUPLE(elem_),                                           \
              __CDEFMT_GENERATE_LOG_ARG_PARAMETER, /* Handle special parameters */ \
              ___CDEFMT_GENERATE_LOG_ARG)          /* Handle regular parameters */ \
  (counter_, i_, elem_);

#define CDEFMT_GENERATE_LOG_ARGS(counter_, args_seq_) \
  BOOST_PP_SEQ_FOR_EACH_I(__CDEFMT_GENERATE_LOG_ARG, counter_, args_seq_)

/* Copies the argument's value into the log struct */
#define CDEFMT_ASSIGN_MEMCPY(counter_, i_, from_)           \
  do {                                                      \
    memcpy(&(CDEFMT_LOG_ARGS(counter_)->arg##i_), &(from_), \
           sizeof(CDEFMT_LOG_ARGS(counter_)->arg##i_));     \
  } while (0)

/* Assign argument to the log buffer */
#define __CDEFMT_ASSIGN_LOG_ARG(r_, counter_, i_, elem_)                          \
  BOOST_PP_IIF(BOOST_VMD_IS_TUPLE(elem_),                                         \
               __CDEFMT_ASSIGN_LOG_ARG_PARAMETER, /* Handle special parameters */ \
               CDEFMT_ASSIGN_MEMCPY)              /* Handle regular parameters */ \
  (counter_, i_, elem_);
#define CDEFMT_ASSIGN_LOG_ARGS(counter_, args_seq_) \
  BOOST_PP_SEQ_FOR_EACH_I(__CDEFMT_ASSIGN_LOG_ARG, counter_, args_seq_)

/* Add size of dynamic parameters to cdefmt_dynamic_size */
#define __CDEFMT_CALCULATE_DYNAMIC_SIZE(r_, counter_, i_, elem_) \
  BOOST_PP_EXPR_IIF(BOOST_VMD_IS_TUPLE(elem_),                   \
                    __CDEFMT_CALCULATE_DYNAMIC_SIZE_PARAMETER(counter_, i_, elem_);)
#define CDEFMT_CALCULATE_DYNAMIC_SIZE(counter_, args_seq_) \
  BOOST_PP_SEQ_FOR_EACH_I(__CDEFMT_CALCULATE_DYNAMIC_SIZE, counter_, args_seq_)

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
        CDEFMT_FORMAT_METADATA(counter_, __CDEFMT_LEVEL_ERR, __FILE__, 0, "cdefmt init: {}", );    \
                                                                                                   \
    struct __attribute__((packed)) CDEFMT_LOG_ARGS_T(counter_) {                                   \
      const char* log_id;                                                                          \
      unsigned char build_id[CDEFMT_GNU_BUILD_ID_SIZE];                                            \
    };                                                                                             \
                                                                                                   \
    struct CDEFMT_LOG_ARGS_T(counter_) CDEFMT_LOG_ARGS(counter_) = {                               \
        .log_id = CDEFMT_LOG_STRING(counter_),                                                     \
    };                                                                                             \
    memcpy(&(CDEFMT_LOG_ARGS(counter_).build_id),                                                  \
           &(__cdefmt_build_id.data[__cdefmt_build_id.name_size]),                                 \
           sizeof(CDEFMT_LOG_ARGS(counter_).build_id));                                            \
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
