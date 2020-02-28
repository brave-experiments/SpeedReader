use super::*;

// NOTE: we use `ExternOutputSink` proxy type, for extern handler function
struct ExternOutputSink {
    handler: unsafe extern "C" fn(*const c_char, size_t),
}

impl ExternOutputSink {
    #[inline]
    fn new(handler: unsafe extern "C" fn(*const c_char, size_t)) -> Self {
        ExternOutputSink { handler }
    }
}

impl OutputSink for ExternOutputSink {
    #[inline]
    fn handle_chunk(&mut self, chunk: &[u8]) {
        let chunk_len = chunk.len();
        let chunk = chunk.as_ptr() as *const c_char;

        unsafe { (self.handler)(chunk, chunk_len) };
    }
}

/// Indicate type of rewriter that would be used based on existing
/// configuration. `RewrtierUnknown` indicates that no configuration was found
/// for the provided parameters.
/// Also used to ask for a specific type of rewriter if desired; passing
/// `RewriterUnknown` tells SpeedReader to look the type up by configuration
/// and use heuristics-based one if not found otherwise.
#[repr(C)]
pub enum CRewriterType {
    RewriterStreaming,
    RewriterHeuristics,
    RewriterUnknown,
}

impl CRewriterType {
    fn to_rewriter_type(&self) -> Option<RewriterType> {
        match &self {
            CRewriterType::RewriterStreaming => Some(RewriterType::Streaming),
            CRewriterType::RewriterHeuristics => Some(RewriterType::Heuristics),
            CRewriterType::RewriterUnknown => None,
        }
    }
}

impl From<RewriterType> for CRewriterType {
    fn from(r_type: RewriterType) -> Self {
        match r_type {
            RewriterType::Streaming => CRewriterType::RewriterStreaming,
            RewriterType::Heuristics => CRewriterType::RewriterHeuristics,
            RewriterType::Unknown => CRewriterType::RewriterUnknown,
        }
    }
}

/// Opaque structure to have the minimum amount of type safety across the FFI.
/// Only replaces c_void
#[repr(C)]
pub struct CSpeedReaderProcessor {
    _private: [u8; 0],
}

#[no_mangle]
pub extern "C" fn speedreader_new() -> *mut SpeedReader {
    to_ptr_mut(SpeedReader::new())
}

#[no_mangle]
pub extern "C" fn speedreader_url_readable(
    speedreader: *mut SpeedReader,
    url: *const c_char,
    url_len: size_t,
) -> bool {
    let url = unwrap_or_ret! { to_str!(url, url_len), false };
    let speedreader = to_ref!(speedreader);
    speedreader.url_readable(url).unwrap_or(false)
}

#[no_mangle]
pub extern "C" fn speedreader_find_type(
    speedreader: *mut SpeedReader,
    url: *const c_char,
    url_len: size_t,
) -> CRewriterType {
    let url = unwrap_or_ret! { to_str!(url, url_len), CRewriterType::RewriterUnknown};
    let speedreader = to_ref!(speedreader);
    let rewriter_type = speedreader.get_rewriter_type(url);
    CRewriterType::from(rewriter_type)
}

/// test documentation
#[no_mangle]
pub extern "C" fn speedreader_get_rewriter(
    speedreader: *mut SpeedReader,
    url: *const c_char,
    url_len: size_t,
    output_sink: unsafe extern "C" fn(*const c_char, size_t),
    rewriter_type: CRewriterType,
) -> *mut CSpeedReaderProcessor {
    let url = unwrap_or_ret_null! { to_str!(url, url_len) };
    let speedreader = to_ref!(speedreader);

    let opaque_config = speedreader.get_opaque_config(url);

    let output_sink = ExternOutputSink::new(output_sink);

    let rewriter = speedreader
        .get_rewriter(
            url,
            &opaque_config,
            output_sink,
            rewriter_type.to_rewriter_type(),
        )
        .unwrap();
    box_to_opaque!(rewriter, CSpeedReaderProcessor)
}

#[no_mangle]
pub extern "C" fn speedreader_free(speedreader: *mut SpeedReader) {
    drop(to_box!(speedreader));
}

#[no_mangle]
pub extern "C" fn speedreader_processor_write(
    processor: *mut CSpeedReaderProcessor,
    chunk: *const c_char,
    chunk_len: size_t,
) -> c_int {
    let chunk = to_bytes!(chunk, chunk_len);
    let processor: &mut Box<dyn SpeedReaderProcessor> = leak_void_to_box!(processor);

    unwrap_or_ret_err_code! { processor.write(chunk) };
    0
}

#[no_mangle]
pub extern "C" fn speedreader_processor_end(processor: *mut CSpeedReaderProcessor) -> c_int {
    let mut processor: Box<Box<dyn SpeedReaderProcessor>> = void_to_box!(processor);
    unwrap_or_ret_err_code! { processor.end() };
    0
}

#[no_mangle]
pub extern "C" fn speedreader_processor_free(processor: *mut CSpeedReaderProcessor) {
    void_to_box!(processor);
}
