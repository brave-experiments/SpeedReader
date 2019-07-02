use readability;
use std::io::Read;
use std::cell::RefCell;
use url::Url;

use crate::classifier;
use classifier::feature_extractor::{FeatureExtractorStreamer};

// refactor out
use html5ever::QualName;
use markup5ever::{Namespace, LocalName, Prefix};

struct SpeedReaderDoc {
    pub readable: bool,
    pub doc: Option<String>
}

fn process<R>(input: &mut R, url: &Url) -> SpeedReaderDoc where R: Read {
    let maybe_featurised = classifier::feature_extractor::FeatureExtractor::parse_document(input, url);
    if maybe_featurised.is_err() {
        eprintln!("Error while processing document: {:?}", maybe_featurised.err());
        return SpeedReaderDoc {
            readable: false,
            doc: None
        }
    }

    let mut featurised = maybe_featurised.unwrap();

    let class = classifier::Classifier::from_feature_map(&featurised.features).classify();

    if class == 0 {
        SpeedReaderDoc {
            readable: false,
            doc: None
        }
    } else {
        let extracted = readability::extractor::extract_dom(&mut featurised.dom, url, &featurised.features).unwrap();
        SpeedReaderDoc {
            readable: true,
            doc: Some(extracted.content)
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
    original_buffer: RefCell<Vec<u8>>,
    readable: RefCell<Option<bool>>,
    streamer: FeatureExtractorStreamer,
}

impl SpeedReader {
    pub fn new(url: &str) -> SpeedReader {
        let url_parsed = Url::parse(url);
        let qn = QualName::new(
            Some(Prefix::from("html")),
            Namespace::from("html"),
            LocalName::from("html"),
        );

        url_parsed.map(|url| {
            if url_maybe_readable(&url) {
                let mut streamer = FeatureExtractorStreamer::new(qn.clone()).unwrap(); 
                streamer.set_url(&url);
                SpeedReader {
                    url: Some(url),
                    original_buffer: RefCell::new(Vec::with_capacity(DOC_CAPACITY_INCREMENTS)),
                    readable: RefCell::new(None),
                    streamer,
                }
            } else {
                SpeedReader {
                    url: None,
                    original_buffer: RefCell::new(Vec::with_capacity(0)),
                    readable: RefCell::new(Some(false)),
                    streamer: FeatureExtractorStreamer::new(qn.clone()).unwrap(),
                }
            }
        })
        .unwrap_or_else(|_e| {
            SpeedReader {
                url: None,
                original_buffer: RefCell::new(Vec::with_capacity(0)),
                readable: RefCell::new(Some(false)),
                streamer: FeatureExtractorStreamer::new(qn.clone()).unwrap(),
            }
        })
    }

    pub fn with_chunk(&mut self, mut input: &mut &[u8]) {
        if self.document_readable() != Some(false) {
            self.streamer.parse_fragment(&mut input);
        }
        // else NOOP - already decided the doc is not readable
    }

    pub fn document_readable(&self) -> Option<bool> {
        *self.readable.borrow()
    }

    pub fn finalize(&self) -> Option<String> {
        // No vlaid URL - no document
        if self.url.is_none() {
            return None;
        }
        // Already decided the document is not readable
        if self.document_readable() == Some(false) {
            return None;
        }
        let processed = process(&mut self.original_buffer.borrow().as_slice(), self.url.as_ref().unwrap());
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

        assert_eq!(sreader.streamer.sink.features["url_depth"], 1);

        sreader.with_chunk(&mut buff1);
        assert_eq!(sreader.streamer.sink.features["p"], 1);

        sreader.with_chunk(&mut buff2);
        assert_eq!(sreader.streamer.sink.features["br"], 2);

        sreader.with_chunk(&mut buff3);
        assert_eq!(sreader.streamer.sink.features["a"], 1);
    }
}
