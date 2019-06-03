extern crate speedreader;

use std::io::Read;
use std::string::String;

use speedreader::feature_extractor::FeatureExtractor;
use std::fs::File;

fn main() {
    let mut data = String::new();
    let mut f = File::open("./examples/html/2CdyGKStt9jwu5u.html").expect("err reading doc");
    f.read_to_string(&mut data).expect("err to string");

    let mut extractor = FeatureExtractor::from_document(data.clone());
    let result = extractor.extract();

    println!("features map len: {}", result.len());
    for (k, v) in result.iter() {
        println!("{}: {}", k, v);
    }
}
