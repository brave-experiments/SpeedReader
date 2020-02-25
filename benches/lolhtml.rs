extern crate url;
extern crate speedreader;
extern crate reqwest;

use url::Url;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lol_html::{HtmlRewriter, Settings};
use lol_html::doc_comments;
use speedreader::streaming::whitelist::Whitelist;
use speedreader::streaming::rewriter_config_builder::RewriterConfigBuilder;
use speedreader::streaming::rewriter_config_builder::get_content_handlers;

fn transform_lol(data: &[u8], url: &Url, output: &mut Vec<u8>, whitelist: &Whitelist) -> Result<(), Box<dyn std::error::Error>> {
    let r_config = RewriterConfigBuilder::new(
        whitelist.get_configuration(&url.domain().unwrap_or_default().replace("www.", "")).unwrap(),
        &url.origin().ascii_serialization(),
    );

    let mut rewriter = HtmlRewriter::try_new(
        Settings {
            element_content_handlers: r_config.handlers
                .iter()
                .map(|(selector, function)| (selector, get_content_handlers(function)))
                .collect(),
            document_content_handlers: vec![doc_comments!(|el| Ok(el.remove()))],
            ..Settings::default()
        },
        black_box(|c: &[u8]| output.extend_from_slice(c))
    )?;

    rewriter.write(data)?;
    rewriter.end()?;

    Ok(())
}

fn transform_html5ever(data: &[u8], url: &Url, output: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {
    let mut sreader = speedreader::speedreader::SpeedReader::try_new(
        url.as_str(),
        |c: &[u8]| {
            output.extend_from_slice(c)
        }
    ).unwrap();

    sreader.write(data);
    sreader.end().ok();

    Ok(())
}

fn bench_lolhtml(c: &mut Criterion) {
    let article_url = "https://www.cnet.com/roadshow/features/2020-acura-nsx-road-trip-daytona/";
    let url = Url::parse(article_url).unwrap();

    let mut whitelist = Whitelist::default();
    whitelist.load_predefined();

    let client = reqwest::blocking::Client::new();
    let data = client.get(article_url)
        .send()
        .unwrap()
        .text()
        .unwrap();

    c.bench_function("lolhtml-cnet", |b| b.iter(|| {
        let mut output = vec![];
        transform_lol(data.as_bytes(), &url, &mut output, &whitelist).unwrap();
    }));
}


fn bench_html5ever(c: &mut Criterion) {
    let article_url = "https://www.cnet.com/roadshow/features/2020-acura-nsx-road-trip-daytona/";
    let url = Url::parse(article_url).unwrap();

    let client = reqwest::blocking::Client::new();
    let data = client.get(article_url)
        .send()
        .unwrap()
        .text()
        .unwrap();

    c.bench_function("html5ever-cnet", |b| b.iter(|| {
        let mut output = vec![];
        transform_html5ever(data.as_bytes(), &url, &mut output).unwrap();
    }));
}

criterion_group!(benches,
    bench_lolhtml,
    bench_html5ever
);
criterion_main!(benches);
