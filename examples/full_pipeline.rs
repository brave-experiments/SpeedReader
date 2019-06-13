extern crate url;
extern crate speedreader;

use speedreader::classifier::feature_extractor::FeatureExtractor;
use readability::extractor::extract_dom;
use speedreader::classifier::Classifier;
use url::Url;
use std::fs;

fn main() {
    let url = Url::parse("http://example.com/hello/world/hello?again").unwrap();
    let data = fs::read_to_string("./examples/html/2CdyGKStt9jwu5u.html").expect("err to string");

    // feature extraction
    let mut extractor = FeatureExtractor::parse_document(&mut data.as_bytes(), &url.to_string());

    println!(">> Feature List");
    for (k, v) in extractor.features.to_owned().iter() {
        println!("{}: {}", k, v);
    }
    
    // document classification
    let classifier_result = Classifier::from_feature_map(&extractor.features)
        .classify();
    println!(">> Readble?\n {}", classifier_result);

    // document mapper
    let product = extract_dom(&mut extractor.dom, &url).unwrap();
    println!(">> Read mode:\n {}", product.content);
}