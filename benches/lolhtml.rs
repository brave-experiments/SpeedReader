extern crate url;
extern crate speedreader;
extern crate reqwest;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_lolhtml(c: &mut Criterion) {
    let article_url = "https://www.cnet.com/roadshow/features/2020-acura-nsx-road-trip-daytona/";

    let client = reqwest::blocking::Client::new();
    let data = client.get(article_url)
        .send()
        .unwrap()
        .text()
        .unwrap();
    
    let sr = speedreader::SpeedReader::new();
    let (config, user_data) = sr.find_config(article_url);

    c.bench_function("lolhtml-cnet", |b| b.iter(|| {
        let mut output = vec![];
        let mut rewriter = sr
            .get_rewriter(article_url, config, &user_data, black_box(|c: &[u8]| output.extend_from_slice(c)))
            .unwrap();
        rewriter.write(data.as_bytes()).ok();
        rewriter.end().ok();
    }));
}


fn bench_html5ever(c: &mut Criterion) {
    let article_url = "https://www.cnet.com/roadshow/features/2020-acura-nsx-road-trip-daytona/";

    let client = reqwest::blocking::Client::new();
    let data = client.get(article_url)
        .send()
        .unwrap()
        .text()
        .unwrap();
    
    let sr = speedreader::SpeedReader::with_whitelist(speedreader::whitelist::Whitelist::default());
    let (config, user_data) = sr.find_config(article_url);

    c.bench_function("html5ever-cnet", |b| b.iter(|| {
        let mut output = vec![];
        let mut rewriter = sr
            .get_rewriter(article_url, config, &user_data, black_box(|c: &[u8]| output.extend_from_slice(c)))
            .unwrap();
        rewriter.write(data.as_bytes()).ok();
        rewriter.end().ok();
    }));
}

criterion_group!(benches,
    bench_lolhtml,
    bench_html5ever
);
criterion_main!(benches);
