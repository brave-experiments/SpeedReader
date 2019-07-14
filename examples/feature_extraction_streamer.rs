extern crate speedreader;

use speedreader::classifier::feature_extractor::{FeatureExtractor, FeatureExtractorStreamer};
use std::fs;
use url::Url;

use html5ever::QualName;

use markup5ever::{Namespace, LocalName, Prefix};

fn main() {
    let url = Url::parse("http://example.com/hello/world/hello?again");
    let frag1 = fs::read_to_string("./examples/html/bbc_new1.html").expect("err to string");
    let frag2 = fs::read_to_string("./examples/html/bbc_new2.html").expect("err to string");
    //let frag3 = fs::read_to_string("./examples/html/frag3.html").expect("err to string");

    // feature extraction with chunker
    let qn = QualName::new(
        Some(Prefix::from("html")),
        Namespace::from("html"),
        LocalName::from("html"),
    );

    let mut streamer = FeatureExtractorStreamer::new(qn).unwrap();
    
    streamer.parse_fragment(&mut frag1.as_bytes());
    streamer.parse_fragment(&mut frag2.as_bytes());
    //streamer.parse_fragment(&mut frag3.as_bytes());
    streamer.set_url(&url.clone().unwrap());

    println!("======\n Features streamer:");
    let result = streamer.sink.features;
    for (k, v) in result.iter() {
        println!("{}: {}", k, v)
    }

    //feature extraction full
    let full = fs::read_to_string("./examples/html/bbc_new.html").expect("err to string");
    let extractor = FeatureExtractor::parse_document(&mut full.as_bytes(), &url.unwrap()).unwrap();

    println!("======\n Features full document:");
    for (l, i) in extractor.features.iter() {
        println!("{}: {}", l,i);
    }
}
