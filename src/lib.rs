#![allow(dead_code)]
#![forbid(unsafe_code)]
extern crate url;
extern crate html5ever;

pub mod classifier;

#[cfg(test)]
#[macro_use]
extern crate matches;

use readability;
use std::io::Read;

// &mut doc.as_bytes()

pub fn process<R>(mut input: &mut R, url: &str) -> String where R: Read {
    let mut featurised = classifier::feature_extractor::FeatureExtractor::parse_document(&mut input, url);
    let class = classifier::Classifier::from_feature_map(&featurised.features).classify();

    if class == 0 {
        "".to_owned() // TODO: return the original content
    } else {
        let extracted = readability::extractor::extract_dom(&mut featurised.dom, &featurised.url, &featurised.features).unwrap();
        extracted.content
    }
}
