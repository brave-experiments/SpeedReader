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
pub struct CSpeedReaderRewriter {
    _private: [u8; 0],
}

/// New instance of SpeedReader. Loads the default configuration and rewriting
/// whitelists. Must be freed by calling `speedreader_free`.
#[no_mangle]
pub extern "C" fn speedreader_new() -> *mut SpeedReader {
    to_ptr_mut(SpeedReader::new())
}

/// Checks if the provided URL matches whitelisted readable URLs.
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

/// Returns type of SpeedReader that would be applied by default for the given
/// URL. `RewriterUnknown` if no match in the whitelist.
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

#[no_mangle]
pub extern "C" fn speedreader_free(speedreader: *mut SpeedReader) {
    drop(to_box!(speedreader));
}

/// Returns SpeedReader rewriter instance for the given URL. If provided
/// `rewriter_type` is `RewriterUnknown`, will look it up in the whitelist
/// and default to heuristics-based rewriter if none found in the whitelist.
/// Returns NULL if no URL provided or initialization fails.
/// Results of rewriting sent to `output_sink` callback function.
/// MUST be finished with `speedreader_rewriter_end` which will free
/// associated memory.
#[no_mangle]
pub extern "C" fn speedreader_rewriter_new(
    speedreader: *mut SpeedReader,
    url: *const c_char,
    url_len: size_t,
    output_sink: unsafe extern "C" fn(*const c_char, size_t),
    rewriter_type: CRewriterType,
) -> *mut CSpeedReaderRewriter {
    let url = unwrap_or_ret_null! { to_str!(url, url_len) };
    let speedreader = to_ref!(speedreader);

    let opaque_config = speedreader.get_opaque_config(url);

    let output_sink = ExternOutputSink::new(output_sink);

    let rewriter = unwrap_or_ret_null! { speedreader
        .get_rewriter(
            url,
            &opaque_config,
            output_sink,
            rewriter_type.to_rewriter_type(),
        )
    };
    box_to_opaque!(rewriter, CSpeedReaderRewriter)
}

/// Write a new chunk of data (byte array) to the rewriter instance.
#[no_mangle]
pub extern "C" fn speedreader_rewriter_write(
    rewriter: *mut CSpeedReaderRewriter,
    chunk: *const c_char,
    chunk_len: size_t,
) -> c_int {
    let chunk = to_bytes!(chunk, chunk_len);
    let rewriter: &mut Box<dyn SpeedReaderProcessor> = leak_void_to_box!(rewriter);

    unwrap_or_ret_err_code! { rewriter.write(chunk) };
    0
}

/// Complete rewriting for this instance.
/// Will free memory used by the rewriter.
/// Calling twice will cause panic.
#[no_mangle]
pub extern "C" fn speedreader_rewriter_end(rewriter: *mut CSpeedReaderRewriter) -> c_int {
    let mut rewriter: Box<Box<dyn SpeedReaderProcessor>> = void_to_box!(rewriter);
    unwrap_or_ret_err_code! { rewriter.end() };
    0
}

#[no_mangle]
pub extern "C" fn speedreader_rewriter_free(rewriter: *mut CSpeedReaderRewriter) {
    void_to_box!(rewriter);
}
