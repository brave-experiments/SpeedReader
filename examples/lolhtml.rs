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

use speedreader::speedreader_streaming::SpeedReaderStreaming;
use speedreader::speedreader::Whitelist;
use speedreader::speedreader::RewriterConfigBuilder;
use speedreader::speedreader::SpeedReaderProcessor;

fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

const USER_AGENT: &'static str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_3) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.106 Safari/537.36";

async fn stream_content(article_url: &str) -> Result<(), Box<dyn Error>> {
    let url = Url::parse(article_url).unwrap();

    let dir = format!(
        "data/lolhtml/{}/{}",
        url.host().unwrap(),
        calculate_hash(&article_url)
    );
    println!("Creating directory: {}", dir);
    fs::create_dir_all(&dir).unwrap_or_default();

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
        &whitelist.get_configuration(&url.domain().unwrap_or_default().replace("www.", ""))
            .unwrap()
            .declarative_rewrite.as_ref()
            .unwrap(),
        &url.origin().ascii_serialization(),
    );

    let mut mapped_file = fs::File::create(format!("{}/mapped.html", &dir))?;
    let mut mapped_test_file = fs::File::create(format!("{}/mapped.html", "data/lolhtml/dump/test"))?;

    let mut rewriter = SpeedReaderStreaming::try_new(
        url,
        |c: &[u8]| {
            mapped_file.write_all(c).ok();
            mapped_test_file.write_all(c).ok();
        },
        &r_config
    ).unwrap();

    let mut init_file = fs::File::create(format!("{}/init.html", &dir))?;
    let mut init_test_file = fs::File::create(format!("{}/init.html", "data/lolhtml/dump/test"))?;
    while let Some(chunk) = data.chunk().await? {
        rewriter.write(chunk.as_ref()).ok();
        init_file.write_all(chunk.as_ref()).ok();
        init_test_file.write_all(chunk.as_ref()).ok();
    }

    rewriter.end().ok();

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let article_url = &args[1];
    stream_content(article_url).await?;

    Ok(())
}
