use readability;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::io::Read;
use url::Url;

use crate::classifier;
use classifier::feature_extractor::{FeatureExtractorStreamer, FeaturisingTreeSink};

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

fn process_full_document<R>(input: &mut R, url: &Url) -> SpeedReaderDoc
where
    R: Read,
{
    let maybe_featurised =
        classifier::feature_extractor::FeatureExtractor::parse_document(input, url);
    if maybe_featurised.is_err() {
        eprintln!(
            "Error while processing document: {:?}",
            maybe_featurised.err()
        );
        return SpeedReaderDoc {
            readable: false,
            doc: None,
        };
    }

    let mut featurised = maybe_featurised.unwrap();

    let class = classifier::Classifier::from_feature_map(&featurised.features).classify();

    if class == 0 {
        SpeedReaderDoc {
            readable: false,
            doc: None,
        }
    } else {
        let extracted =
            readability::extractor::extract_dom(&mut featurised.dom, url, &featurised.features)
                .unwrap();
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

const DOC_CAPACITY_INCREMENTS: usize = 65536;

pub struct SpeedReader {
    url: Option<Url>,
    readable: RefCell<Option<bool>>,
    streamer: Option<FeatureExtractorStreamer>,
}

impl SpeedReader {
    pub fn new(url: &str) -> SpeedReader {
        let url_parsed = Url::parse(url);

        url_parsed
            .map(|url| {
                if url_maybe_readable(&url) {
                    let streamer = FeatureExtractorStreamer::new(&url).ok();
                    SpeedReader {
                        url: Some(url),
                        readable: RefCell::new(None),
                        streamer,
                    }
                } else {
                    SpeedReader {
                        url: None,
                        readable: RefCell::new(Some(false)),
                        streamer: None,
                    }
                }
            })
            .unwrap_or_else(|_e| SpeedReader {
                url: None,
                readable: RefCell::new(Some(false)),
                streamer: None,
            })
    }

    pub fn with_chunk(&mut self, input: &[u8]) {
        if self.document_readable() != Some(false) && self.streamer.is_some() {
            let streamer = self.streamer.as_mut().unwrap();
            match streamer.write(&mut input.borrow()) {
                Err(_) => *self.readable.borrow_mut() = Some(false),
                _ => (),
            }
        }
        // else NOOP - already decided the doc is not readable
    }

    pub fn document_readable(&self) -> Option<bool> {
        *self.readable.borrow()
    }

    pub fn finalize(&mut self) -> Option<String> {
        // No valid URL - no document
        if self.url.is_none() || self.streamer.is_none() {
            return None;
        }
        // Already decided the document is not readable
        if self.document_readable() == Some(false) {
            return None;
        }
        let url = self.url.as_ref().unwrap();
        let streamer = self.streamer.as_mut().unwrap();
        let processed = process(streamer.finish(), url);

        *self.readable.borrow_mut() = Some(processed.readable);
        if processed.readable {
            processed.doc
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_speedreader_streamer() {
        let mut sreader = SpeedReader::new("https://test.xyz");

        let mut buff1 = "<html><p>hello".as_bytes();
        let mut buff2 = "world </p>\n\n\n\n<br><br><a href='/link'>".as_bytes();
        let mut buff3 = "this is a link</a></html>".as_bytes();

        sreader.with_chunk(&mut buff1);
        sreader.with_chunk(&mut buff2);
        sreader.with_chunk(&mut buff3);
        let result_sink = sreader.streamer.as_mut().unwrap().finish();

        assert_eq!(result_sink.features["url_depth"], 1);
        assert_eq!(result_sink.features["p"], 1);
        assert_eq!(result_sink.features["a"], 1);
    }
}
