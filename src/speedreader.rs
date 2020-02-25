use readability;
use std::borrow::Borrow;
use std::cell::RefCell;
use url::Url;

use crate::classifier;
use classifier::feature_extractor::{FeatureExtractorStreamer, FeaturisingTreeSink};
use classifier::feature_extractor::FeatureExtractorError;

/// Defines an interface for the [`HtmlRewriter`]'s output.
///
/// Implemented for [`Fn`] and [`FnMut`].
///
/// [`HtmlRewriter`]: struct.HtmlRewriter.html
/// [`Fn`]: https://doc.rust-lang.org/std/ops/trait.Fn.html
/// [`FnMut`]: https://doc.rust-lang.org/std/ops/trait.FnMut.html
pub trait OutputSink {
    /// Handles rewriter's output chunk.
    ///
    /// # Note
    /// The last chunk of the output has zero length.
    fn handle_chunk(&mut self, chunk: &[u8]);
}

impl<F: FnMut(&[u8])> OutputSink for F {
    fn handle_chunk(&mut self, chunk: &[u8]) {
        self(chunk);
    }
}

pub struct SpeedReader<O>
where
    O: OutputSink
{
    url: Option<Url>,
    readable: RefCell<Option<bool>>,
    streamer: FeatureExtractorStreamer,
    output_sink: O,
}

impl<O: OutputSink> SpeedReader<O> {
    pub fn try_new(url: &str, output_sink: O) -> Result<Self, FeatureExtractorError> {
        let url_parsed = Url::parse(url);

        url_parsed
            .map(|url_parsed| {
                if url_maybe_readable(&url_parsed) {
                    let streamer = FeatureExtractorStreamer::try_new(&url_parsed)?;
                    Ok(SpeedReader {
                        url: Some(url_parsed),
                        readable: RefCell::new(None),
                        streamer,
                        output_sink
                    })
                } else {
                    Err(FeatureExtractorError::InvalidUrl(url.to_owned()))
                }
            })?
    }

    pub fn write(&mut self, input: &[u8]) {
        if self.document_readable() != Some(false) {
            match self.streamer.write(&mut input.borrow()) {
                Err(_) => *self.readable.borrow_mut() = Some(false),
                _ => (),
            }
        }
        // else NOOP - already decided the doc is not readable
    }

    pub fn document_readable(&self) -> Option<bool> {
        *self.readable.borrow()
    }

    pub fn end(&mut self) -> Result<(), FeatureExtractorError> {
        // No valid URL - no document
        if self.url.is_none() {
            return Err(FeatureExtractorError::InvalidUrl("".to_owned()));
        }
        // Already decided the document is not readable
        if self.document_readable() == Some(false) {
            return Err(FeatureExtractorError::DocumentParseError("Not readable".to_owned()));
        }
        let url = self.url.as_ref().unwrap();
        let processed = process(self.streamer.end(), url);

        *self.readable.borrow_mut() = Some(processed.readable);
        if processed.readable {
            if let Some(doc) = processed.doc {
                self.output_sink.handle_chunk(doc.as_bytes());
                Ok(())
            } else {
                Err(FeatureExtractorError::DocumentParseError("Not readable".to_owned()))
            }
            
        } else {
            Err(FeatureExtractorError::DocumentParseError("Not readable".to_owned()))
        }
    }
}

struct SpeedReaderDoc {
    pub readable: bool,
    pub doc: Option<String>,
}


fn process(sink: &mut FeaturisingTreeSink, url: &Url) -> SpeedReaderDoc {
    let class = classifier::Classifier::from_feature_map(&sink.features).classify();
    if class == 0 {
        SpeedReaderDoc {
            readable: false,
            doc: None,
        }
    } else {
        let extracted =
            readability::extractor::extract_dom(&mut sink.rcdom, url, &sink.features).unwrap();
        SpeedReaderDoc {
            readable: true,
            doc: Some(extracted.content),
        }
    }
}

fn url_maybe_readable(url: &Url) -> bool {
    let scheme = url.scheme();
    scheme == "http" || scheme == "https"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speedreader_streamer() {
        let mut buf = vec![];
        let mut sreader = SpeedReader::try_new(
            "https://test.xyz",
            |c: &[u8]| {
                buf.extend_from_slice(c)
            }
        ).unwrap();

        let buff1 = "<html><p>hello".as_bytes();
        let buff2 = "world </p>\n\n\n\n<br><br><a href='/link'>".as_bytes();
        let buff3 = "this is a link</a></html>".as_bytes();

        sreader.write(&buff1);
        sreader.write(&buff2);
        sreader.write(&buff3);
        sreader.end().ok();
        let result_sink = sreader.streamer.end();

        assert_eq!(result_sink.features["url_depth"], 1);
        assert_eq!(result_sink.features["p"], 1);
        assert_eq!(result_sink.features["a"], 1);
    }
}
