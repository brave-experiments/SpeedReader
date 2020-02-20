extern crate url;
extern crate speedreader;
extern crate reqwest;

use url::Url;

use speedreader::classifier::feature_extractor::FeatureExtractor;
use readability::extractor::extract_dom;
use speedreader::classifier::Classifier;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lol_html::{element, HtmlRewriter, Settings};
use lol_html::{text};
use lol_html::html_content::ContentType;
use lol_html::html_content::UserData;

fn map(data: &[u8], output: &mut Vec<u8>) -> Result<(), Box<dyn std::error::Error>> {

    let mut rewriter = HtmlRewriter::try_new(
        Settings {
            element_content_handlers: vec![
                element!(".article-main-body *", |el| {
                    el.set_user_data(true);
                    Ok(())
                }),
                text!(".article-main-body *", |el| {
                    el.set_user_data(true);
                    Ok(())
                }),
                text!("*", |t| {
                    let user_data = t.user_data_mut().downcast_ref::<bool>();
                    if user_data != Some(&true) {
                        t.remove();
                    }
                    Ok(())
                }),
                element!("*", |el| {
                    let user_data = el.user_data_mut().downcast_ref::<bool>();
                    if user_data != Some(&true) {
                        el.remove_and_keep_content();
                    }
                    Ok(())
                }),
                // element!("script", |el| {
                //     el.remove();
                //     Ok(())
                // }),
                // element!("meta", |el| {
                //     el.remove();
                //     Ok(())
                // }),
                // element!("link", |el| {
                //     el.remove();
                //     Ok(())
                // }),
                
                element!("picture source", |el| {
                    el.set_attribute("width", "100%")?;
                    el.set_attribute("height", "auto")?;
                    Ok(())
                }),
                element!("picture img", |el| {
                    el.set_attribute("width", "100%")?;
                    el.set_attribute("height", "auto")?;
                    Ok(())
                }),
                element!("figure.image img", |el| {
                    el.set_attribute("width", "100%")?;
                    el.set_attribute("height", "auto")?;
                    el.get_attribute("data-original").map(|src|{
                        el.set_attribute("src", &src).ok();
                    });
                    Ok(())
                }),
                // element!(".twitterContainer", |el| {
                //     el.prepend(r#"<script type="text/javascript" class="optanon-category-5" id="script_twitterwidget" src="//platform.twitter.com/widgets.js" async=""></script>"#, ContentType::Html);
                //     Ok(())  
                // }),
                
            ],
            ..Settings::default()
        },
        |c: &[u8]| output.extend_from_slice(c)
    )?;

    rewriter.write(data)?;
    rewriter.end()?;

    Ok(())
}

fn transformhtml5ever(mut data: &mut &[u8], url: &Url, output: &mut Vec<u8>) {
    // feature extraction
    let mut extractor = FeatureExtractor::parse_document(&mut data, url).unwrap();
    
    // document classification
    let classifier_result = Classifier::from_feature_map(&extractor.features)
        .classify();

    if classifier_result > 0 {
        // document mapper
        let product = extract_dom(&mut extractor.dom, &url, &extractor.features).unwrap();
        &output.extend_from_slice(product.content.as_bytes());
    }
}

fn bench_lolhtml(c: &mut Criterion) {
    let article_url = "https://www.cnet.com/roadshow/features/2020-acura-nsx-road-trip-daytona/";

    // let url = Url::parse(article_url).unwrap();

    let client = reqwest::blocking::Client::new();
    let data = client.get(article_url)
        .send()
        .unwrap()
        .text()
        .unwrap();

    c.bench_function("lolhtml-cnet", |b| b.iter(|| {
        let mut output = vec![];
        map(data.as_bytes(), &mut output).unwrap();
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
        transformhtml5ever(&mut data.as_bytes(), &url, &mut output);
    }));
}

criterion_group!(benches,
    bench_lolhtml,
    bench_html5ever
);
criterion_main!(benches);
