use std::io::Read;
use std::collections::BTreeMap;
use std::path::Path;
use std::cell::Cell;
use html5ever::rcdom::{RcDom};
use html5ever::{parse_document, serialize};
use html5ever::tendril::stream::TendrilSink;
use std::default::Default;
use url::Url;
use error::Error;
use dom;
use scorer;
use scorer::Candidate;

#[derive(Debug)]
pub struct Product {
    pub title:     String,
    pub content:   String,
    pub text:      String,
}

pub fn extract<R>(input: &mut R, url: &Url) -> Result<Product, Error> where R: Read {
    let mut dom = parse_document(RcDom::default(), Default::default())
        .from_utf8()
        .read_from(input)
        .unwrap();

    extract_dom(&mut dom, url)
}

pub fn extract_dom(mut dom: &mut RcDom, url: &Url) -> Result<Product, Error> {
    let mut title      = String::new();
    let mut candidates = BTreeMap::new();
    let mut nodes      = BTreeMap::new();
    let handle = dom.document.clone();

    // extracts title (if it exists) pre-processes the DOM by removing script
    // tags, css, links
    scorer::preprocess(&mut dom, handle.clone(), &mut title);   
    scorer::find_candidates(&mut dom, Path::new("/"), handle.clone(), &mut candidates, &mut nodes);
    let mut id: &str = "/";
    let mut top_candidate: &Candidate = &Candidate {
        node:  handle.clone(),
        score: Cell::new(0.0),
    };
    for (i, c) in candidates.iter() {
        let score = c.score.get() * (1.0 - scorer::get_link_density(c.node.clone()));
        c.score.set(score);
        if score <= top_candidate.score.get() {
            continue;
        }
        id            = i;
        top_candidate = c;
    }
    let mut bytes = vec![];

    let node = top_candidate.node.clone();
    scorer::clean(&mut dom, Path::new(id), node.clone(), url, &candidates);

    serialize(&mut bytes, &node, Default::default()).ok();
    let content = String::from_utf8(bytes).unwrap_or_default();

    let mut text: String = String::new();
    dom::extract_text(node.clone(), &mut text, true);
    Ok(Product { title: title, content: content, text: text })
}


#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use url::Url;

    #[test]
    fn test_extract_title() {
        assert!(true);
        let mut file = File::open("./tests/samples/simple_title/title.html").unwrap();
        let url = Url::parse("https://example.com").unwrap();
        let product = extract(&mut file, &url).unwrap();
        assert_eq!(product.title, "This is title");
    }
}
