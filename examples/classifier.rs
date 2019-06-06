use std::io::Read;
use std::string::String;

use speedreader::classifier::Classifier;
use speedreader::feature_extractor::FeatureExtractor;
use std::fs::File;

fn main() {
    let url = String::from("http://example.com/hello/world/hello?again");
    let doc_path = "./examples/html/2CdyGKStt9jwu5u.html";
    //let doc_path = "./examples/html/gp-ex2.html";
    //let doc_path = "./examples/html/gp-index.html";
    //let doc_path = "./examples/html/simple.html";

    let mut data = String::new();
    let mut f = File::open(doc_path).expect("err reading doc");
    f.read_to_string(&mut data).expect("err to string");

    let mut extractor = FeatureExtractor::from_document(data, url);
    let features = extractor.extract();

    let result = Classifier::from_feature_map(features).classify();
    println!("{}", result);
}
