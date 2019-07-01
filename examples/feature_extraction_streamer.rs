extern crate speedreader;

use std::io::{self, Write};
use std::str;
use speedreader::classifier::feature_extractor::{FeatureExtractor, FeatureExtractorStreamer};
use std::fs;
use url::Url;

use html5ever::{parse_document, parse_fragment, serialize, QualName};
use html5ever::driver::ParseOpts;
use html5ever::rcdom::RcDom;

use html5ever::tendril::{TendrilSink, Tendril};
use markup5ever::{Namespace, LocalName, Prefix};

fn main() {
    let url = Url::parse("http://example.com/hello/world/hello?again");
    let frag1 = fs::read_to_string("./examples/html/frag1.html").expect("err to string");
    let frag2 = fs::read_to_string("./examples/html/frag2.html").expect("err to string");
    let frag3 = fs::read_to_string("./examples/html/frag3.html").expect("err to string");

    // feature extraction with chunker
    let qn = QualName::new(
        Some(Prefix::from("html")),
        Namespace::from("html"),
        LocalName::from("html"),
    );

    let mut streamer = FeatureExtractorStreamer::new(qn, &url).unwrap();
    
    streamer.parse_fragment(&mut frag1.as_bytes());
    streamer.parse_fragment(&mut frag2.as_bytes());
    streamer.parse_fragment(&mut frag3.as_bytes());
    
    println!("======\n Features streamer:");
    let result = streamer.sink.features;
    for (k, v) in result.iter() {
        println!("{}: {}", k, v)
    }

    //feature extraction full
    let full = fs::read_to_string("./examples/html/simple.html").expect("err to string");
    let extractor = FeatureExtractor::parse_document(&mut full.as_bytes(), &url).unwrap();

    println!("======\n Features full document:");
    for (l, i) in result.iter() {
        println!("{}: {}", l,i);
    }


}
