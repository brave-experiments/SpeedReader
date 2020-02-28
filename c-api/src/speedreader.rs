use super::*;
use libc::c_void;

// NOTE: we use `ExternOutputSink` proxy type, because we need an
// existential type parameter for the `HtmlRewriter` and FnMut can't
// be used as such since it's a trait.
pub struct ExternOutputSink {
    handler: unsafe extern "C" fn(*const c_char, size_t)
}

impl ExternOutputSink {
    #[inline]
    fn new(
        handler: unsafe extern "C" fn(*const c_char, size_t),
    ) -> Self {
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

#[repr(C)]
pub enum CRewriterType {
    RewriterStreaming,
    RewriterHeuristics,
    RewriterUnknown,
}

impl CRewriterType {
    fn to_rewriter_type(&self) -> RewriterType {
        match &self {
            CRewriterType::RewriterStreaming => RewriterType::Streaming,
            CRewriterType::RewriterHeuristics => RewriterType::Heuristics,
            CRewriterType::RewriterUnknown => RewriterType::Unknown,
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
    let (rewriter_type, _) = speedreader.get_rewriter_type(url);
    CRewriterType::from(rewriter_type)
}

#[no_mangle]
pub extern "C" fn speedreader_find_config_extras(
    speedreader: *mut SpeedReader,
    url: *const c_char,
    url_len: size_t,
) -> *mut c_void {
    let url = unwrap_or_ret_null! { to_str!(url, url_len) };
    let speedreader = to_ref!(speedreader);
    let (_, opaque) = speedreader.get_rewriter_type(url);
    to_ptr(opaque) as *mut c_void
}

#[no_mangle]
pub extern "C" fn speedreader_get_rewriter(
    speedreader: *mut SpeedReader,
    url: *const c_char,
    url_len: size_t,
    rewriter_type: CRewriterType,
    config_extras: *mut c_void,
    output_sink: unsafe extern "C" fn(*const c_char, size_t),
    // output_sink_user_data: *mut c_void,
) -> *mut c_void {
    let url = unwrap_or_ret_null! { to_str!(url, url_len) };
    let speedreader = to_ref!(speedreader);

    let config_extras: Box<Box<_>> = void_to_box!(config_extras);

    let output_sink = ExternOutputSink::new(output_sink);

    let rewriter = speedreader
        .get_rewriter(
            url,
            rewriter_type.to_rewriter_type(),
            &config_extras,
            output_sink,
        )
        .unwrap();
    
    to_ptr(rewriter) as *mut c_void
}

#[no_mangle]
pub extern "C" fn speedreader_free(speedreader: *mut SpeedReader) {
    drop(to_box!(speedreader));
}

#[no_mangle]
pub extern "C" fn speedreader_processor_write(
    processor: *mut c_void,
    chunk: *const c_char,
    chunk_len: size_t,
) -> c_int {
    let chunk = to_bytes!(chunk, chunk_len);
    let mut processor: Box<Box<dyn SpeedReaderProcessor>> = void_to_box!(processor);

    unwrap_or_ret_err_code! { processor.write(chunk) };

    0
}

#[no_mangle]
pub extern "C" fn speedreader_processor_end(processor: *mut c_void) -> c_int {
    let mut processor: Box<Box<dyn SpeedReaderProcessor>> = void_to_box!(processor);

    unwrap_or_ret_err_code! { processor.end() };

    0
}

#[no_mangle]
pub extern "C" fn speedreader_processor_free(processor: *mut c_void, config_extras: *mut c_void) {
    // let processor: Box<Box<dyn SpeedReaderProcessor>> =
    //     unsafe { Box::from_raw(processor as *mut _) };
    // drop(processor.as_ref().as_ref());

    // let config: Box<Box<dyn Any>> =
    //     unsafe { Box::from_raw(config_extras as *mut _) };
    // drop(config.as_ref().as_ref());
}
