#include <cstdarg>
#include <cstdint>
#include <cstdlib>
#include <new>

/// Indicate type of rewriter that would be used based on existing
/// configuration. `RewrtierUnknown` indicates that no configuration was found
/// for the provided parameters.
/// Also used to ask for a specific type of rewriter if desired; passing
/// `RewriterUnknown` tells SpeedReader to look the type up by configuration
/// and use heuristics-based one if not found otherwise.
enum class CRewriterType {
  RewriterStreaming,
  RewriterHeuristics,
  RewriterUnknown,
};

/// Opaque structure to have the minimum amount of type safety across the FFI.
/// Only replaces c_void
struct CSpeedReaderProcessor {
  uint8_t _private[0];
};

extern "C" {

CRewriterType speedreader_find_type(SpeedReader *speedreader, const char *url, size_t url_len);

void speedreader_free(SpeedReader *speedreader);

/// test documentation
CSpeedReaderProcessor *speedreader_get_rewriter(SpeedReader *speedreader,
                                                const char *url,
                                                size_t url_len,
                                                void (*output_sink)(const char*, size_t),
                                                CRewriterType rewriter_type);

SpeedReader *speedreader_new();

int speedreader_processor_end(CSpeedReaderProcessor *processor);

void speedreader_processor_free(CSpeedReaderProcessor *processor);

int speedreader_processor_write(CSpeedReaderProcessor *processor,
                                const char *chunk,
                                size_t chunk_len);

bool speedreader_url_readable(SpeedReader *speedreader, const char *url, size_t url_len);

} // extern "C"
