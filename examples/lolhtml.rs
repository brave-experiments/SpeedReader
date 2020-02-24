extern crate reqwest;
extern crate speedreader;
extern crate url;

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use url::Url;

use lol_html::doc_comments;
use lol_html::{HtmlRewriter, Settings};

use speedreader::streaming::whitelist::Whitelist;
use speedreader::streaming::speedreader_streamer::SpeedReader;
use speedreader::streaming::speedreader_streamer::*;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let article_url = &args[1];

    let url = Url::parse(article_url).unwrap();

    let client = reqwest::blocking::Client::new();
    let data = client
        .get(article_url)
        .header("cookie", "")
        .header("user-agent", "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.106 Safari/537.36")
        .send()
        .unwrap()
        .text()
        .unwrap();

    let dir = format!(
        "data/lolhtml/{}/{}",
        url.host().unwrap(),
        calculate_hash(article_url)
    );
    println!("Creating directory: {}", dir);
    fs::create_dir_all(&dir).unwrap_or_default();

    fs::File::create(format!("{}/init.html", &dir))?
        .write_all(data.as_bytes())?;

    fs::File::create(format!("{}/init.html", "data/lolhtml/dump/test"))?
        .write_all(data.as_bytes())?;

    let mut output = vec![];
    
    let mut whitelist = Whitelist::default();
    whitelist.load_predefined();
    let config = whitelist.get_configuration("reuters.com").unwrap();
    println!("Got config: {:?}", &config);
    
    let sr = SpeedReader::new(
        config,
        &url.origin().ascii_serialization(),
    );

    println!("Got handler: {:?}", &sr.handlers.iter().map(|h| &h.0).collect::<Vec<_>>());

    let mut rewriter = HtmlRewriter::try_new(
        Settings {
            element_content_handlers: sr.handlers
                .iter()
                .map(|(selector, function)| (selector, get_content_handlers(function)))
                .collect(),
            document_content_handlers: vec![doc_comments!(|el| Ok(el.remove()))],
            ..Settings::default()
        },
        |c: &[u8]| {
            println!("processed chunk... {}", c.len());
            output.extend_from_slice(c)
        }
    ).unwrap();
    
    rewriter.write(data.as_bytes())?;
    rewriter.end()?;
    
    fs::File::create(format!("{}/mapped.html", &dir))?
        .write_all(&output)?;

    fs::File::create(format!("{}/mapped.html", "data/lolhtml/dump/test"))?
        .write_all(&output)?;
    Ok(())
}
