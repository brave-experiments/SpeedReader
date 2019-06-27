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
    let url = Url::parse("http://example.com/hello/world/hello?again").unwrap();
    let frag1 = fs::read_to_string("./examples/html/frag1.html").expect("err to string");
    let frag2 = fs::read_to_string("./examples/html/frag2.html").expect("err to string");
    let frag3 = fs::read_to_string("./examples/html/frag3.html").expect("err to string");

    let qn = QualName::new(
        Some(Prefix::from("html")),
        Namespace::from("html"),
        LocalName::from("html"),
    );

    let streamer = FeatureExtractorStreamer::new(qn, &url).unwrap();
    
    streamer.parse_fragment(&mut frag1.as_bytes());
    streamer.parse_fragment(&mut frag2.as_bytes());
    streamer.parse_fragment(&mut frag3.as_bytes());
    
    let result = streamer.features;
    for (k, v) in result.iter() {
        println!("{}: {}", k, v)
    }
}

//fn main() {
//    let url = "http://example.com/hello/world/hello?again";
//    let frag1_location = "./examples/html/frag1.html";
//    let frag2_location = "./examples/html/frag2.html";
//    let frag3_location = "./examples/html/frag3.html";
//
//    let frag1 = fs::read_to_string(frag1_location).expect("err to string");
//    let frag2 = fs::read_to_string(frag2_location).expect("err to string");
//    let frag3 = fs::read_to_string(frag3_location).expect("err to string");
//
//    let qual = QualName::new(
//        Some(Prefix::from("html")),
//        Namespace::from("html"),
//        LocalName::from("html"),
//    );
//
//    let mut sink = RcDom::default();    // RcDom<T>
//    let dom = parse_fragment(       // Parser<Sink>
//        sink,
//        ParseOpts::default(),
//        qual.clone(),
//        vec![]);
//    
//    sink = dom.from_utf8()
//        .read_from(&mut frag1.as_bytes()).expect(""); // RcDom<T>
//
//    let dom2 = parse_fragment(
//        sink,
//        ParseOpts::default(),
//        qual.clone(),
//        vec![]);
// 
//    sink = dom2.from_utf8()
//        .read_from(&mut frag2.as_bytes()).expect(""); // RcDom<T>
//
//    let dom3 = parse_fragment(
//        sink,
//        ParseOpts::default(),
//        qual.clone(),
//        vec![]);
// 
//    sink = dom3.from_utf8()
//        .read_from(&mut frag3.as_bytes()).expect(""); // RcDom<T>
//
//    io::stdout()
//        .write_all(b"<!DOCTYPE html>\n")
//        .ok().expect("writing DOCTYPE failed");
//        
//    serialize(&mut io::stdout(), &sink.document, Default::default())
//        .ok().expect("serialization failed");
//}
