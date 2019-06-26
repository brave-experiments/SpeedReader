use speedreader::classifier::feature_extractor::FeatureExtractor;
use speedreader::classifier::Classifier;
use std::fs;
use url::Url;

fn main() {
    let url = Url::parse("http://example.com/hello/world/hello?again").unwrap();
    let doc_path = "./examples/html/2CdyGKStt9jwu5u.html";
    //let doc_path = "./examples/html/gp-ex2.html";
    //let doc_path = "./examples/html/gp-index.html";
    //let doc_path = "./examples/html/simple.html";

    let data = fs::read_to_string(doc_path).expect("err to string");

    let extractor = FeatureExtractor::parse_document(&mut data.as_bytes(), &url).unwrap();
    let features = extractor.features;

    let result = Classifier::from_feature_map(&features).classify();
    println!("{}", result);
}
