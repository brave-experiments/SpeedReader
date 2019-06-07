extern crate speedreader;

use speedreader::classifier::feature_extractor::FeatureExtractor;
use std::fs;

fn main() {
    let url = "http://example.com/hello/world/hello?again";
    let doc_path = "./examples/html/2CdyGKStt9jwu5u.html";
    //let doc_path = "./examples/html/gp-ex2.html";
    //let doc_path = "./examples/html/simple.html";

    let data = fs::read_to_string(doc_path).expect("err to string");

    let extractor = FeatureExtractor::parse_document(&mut data.as_bytes(), url);
    let result = extractor.features;

    for (k, v) in result.iter() {
        println!("{}: {}", k, v);
    }
}
