extern crate speedreader;

use std::io::Read;
use std::string::String;

use speedreader::feature_extractor_tokenizer::FeatureExtractorT;
use std::fs::File;

fn main() {
    let url = String::from("http://example.com/hello/world/hello?again");
    let doc_path = "./examples/html/2CdyGKStt9jwu5u.html";
    //let doc_path = "./examples/html/gp-ex2.html";
    //let doc_path = "./examples/html/simple.html";

    let mut data = String::new();
    let mut f = File::open(doc_path).expect("err reading doc");
    f.read_to_string(&mut data).expect("err to string");

    let mut extractor = FeatureExtractorT::from_document(data, url);
    let result = extractor.extract();

    println!("features map len: {}\n", result.len());
    for (k, v) in result.iter() {
        println!("{}: {}", k, v);
    }
}
