use readability;
use std::io::Read;

use crate::classifier;

pub struct SpeedReaderDoc {
    pub readable: bool,
    pub doc: Option<String>
}

pub fn process<R>(mut input: &mut R, url: &str) -> SpeedReaderDoc where R: Read {
    let maybe_featurised = classifier::feature_extractor::FeatureExtractor::parse_document(&mut input, url);
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
        let extracted = readability::extractor::extract_dom(&mut featurised.dom, &featurised.url).unwrap();
        SpeedReaderDoc {
            readable: true,
            doc: Some(extracted.content)
        }
    }
}
