#ifndef SPEEDREADER_RUST_FFI_H
#define SPEEDREADER_RUST_FFI_H

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  RewriterStreaming,
  RewriterHeuristics,
  RewriterUnknown,
} C_CRewriterType;

typedef struct C_SpeedReader C_SpeedReader;

void *speedreader_find_config_extras(C_SpeedReader *speedreader, const char *url, size_t url_len);

C_CRewriterType speedreader_find_type(C_SpeedReader *speedreader, const char *url, size_t url_len);

void speedreader_free(C_SpeedReader *speedreader);

void *speedreader_get_rewriter(C_SpeedReader *speedreader,
                               const char *url,
                               size_t url_len,
                               C_CRewriterType rewriter_type,
                               void *config_extras,
                               void (*output_sink)(const char*, size_t));

C_SpeedReader *speedreader_new(void);

int speedreader_processor_end(void *processor);

void speedreader_processor_free(void *processor, void *config_extras);

int speedreader_processor_write(void *processor, const char *chunk, size_t chunk_len);

bool speedreader_url_readable(C_SpeedReader *speedreader, const char *url, size_t url_len);

#endif /* SPEEDREADER_RUST_FFI_H */
