use url::Url;
use lol_html::doc_comments;
use lol_html::{HtmlRewriter, Settings};
use lol_html::OutputSink;

use super::speedreader::*;

pub struct SpeedReaderStreaming<'h, O> where O: OutputSink {
    _url: Url,
    rewriter: HtmlRewriter<'h, O>
}

impl<'h, O: OutputSink> SpeedReaderProcessor for SpeedReaderStreaming<'h, O> {
    fn write(&mut self, chunk: &[u8]) -> Result<(), SpeedReaderError> {
        &self.rewriter.write(chunk)?;
        Ok(())
    }

    fn end(&mut self) -> Result<(), SpeedReaderError> {
        &self.rewriter.end()?;
        Ok(())
    }
}

impl<'h, O: OutputSink> SpeedReaderStreaming<'h, O> {
    pub fn try_new(url: Url, output_sink: O, config: &'h RewriterConfigBuilder) -> Result<Self, SpeedReaderError> {
        let mut whitelist = Whitelist::default();
        whitelist.load_predefined();
    
        let rewriter = HtmlRewriter::try_new(
            Settings {
                element_content_handlers: content_handlers(&config.handlers),
                document_content_handlers: vec![doc_comments!(|el| Ok(el.remove()))],
                ..Settings::default()
            },
            output_sink
        )?;


        let sr = SpeedReaderStreaming {
            _url: url,
            rewriter: rewriter
        };

        Ok(sr)
    }
}
