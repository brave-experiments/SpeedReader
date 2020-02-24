extern crate reqwest;
extern crate speedreader;
extern crate url;

use std::collections::hash_map::DefaultHasher;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io::prelude::*;
use std::error::Error;
use url::Url;

use lol_html::doc_comments;
use lol_html::{HtmlRewriter, Settings};

use speedreader::streaming::whitelist::Whitelist;
use speedreader::streaming::rewriter_config_builder::RewriterConfigBuilder;
use speedreader::streaming::rewriter_config_builder::get_content_handlers;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.106 Safari/537.36";

async fn stream_content(url: &Url, dir: &str) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut data = client
        .get(url.as_str())
        .header("cookie", "")
        .header("user-agent", USER_AGENT)
        .send()
        .await?;

    let mut whitelist = Whitelist::default();
    whitelist.load_predefined();
    
    let r_config = RewriterConfigBuilder::new(
        whitelist.get_configuration(&url.domain().unwrap_or_default().replace("www.", "")).unwrap(),
        &url.origin().ascii_serialization(),
    );

    let mut mapped_file = fs::File::create(format!("{}/mapped.html", &dir))?;
    let mut mapped_test_file = fs::File::create(format!("{}/mapped.html", "data/lolhtml/dump/test"))?;

    let mut rewriter = HtmlRewriter::try_new(
        Settings {
            element_content_handlers: r_config.handlers
                .iter()
                .map(|(selector, function)| (selector, get_content_handlers(function)))
                .collect(),
            document_content_handlers: vec![doc_comments!(|el| Ok(el.remove()))],
            ..Settings::default()
        },
        |c: &[u8]| {
            mapped_file.write_all(c).ok();
            mapped_test_file.write_all(c).ok();
        }
    ).unwrap();

    let mut init_file = fs::File::create(format!("{}/init.html", &dir))?;
    let mut init_test_file = fs::File::create(format!("{}/init.html", "data/lolhtml/dump/test"))?;
    while let Some(chunk) = data.chunk().await? {
        rewriter.write(chunk.as_ref())?;
        init_file.write_all(chunk.as_ref())?;
        init_test_file.write_all(chunk.as_ref())?;
    }

    rewriter.end()?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();
    let article_url = &args[1];

    let url = Url::parse(article_url).unwrap();

    let dir = format!(
        "data/lolhtml/{}/{}",
        url.host().unwrap(),
        calculate_hash(article_url)
    );
    println!("Creating directory: {}", dir);
    fs::create_dir_all(&dir).unwrap_or_default();

    
    stream_content(&url, &dir).await?;

    Ok(())
}
