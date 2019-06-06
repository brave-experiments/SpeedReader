use std::borrow::Cow;
use std::collections::HashMap;
use std::default::Default;
use std::string::String;
use std::vec::Vec;

use html5ever::parse_document;
use html5ever::tendril::*;
use html5ever::tree_builder::{
    AppendNode, AppendText, ElementFlags, NodeOrText, QuirksMode, TreeSink,
};
use html5ever::{Attribute, ExpandedName, QualName};

use regex::Regex;

pub struct FeatureExtractor {
    doc: String,
    url: String,
}

impl FeatureExtractor {
    pub fn from_document(doc: String, url: String) -> FeatureExtractor {
        FeatureExtractor { doc, url }
    }

    pub fn extract(&mut self) -> HashMap<String, usize> {
        let mut features = self.process_and_extract();
        features.insert("url_depth".to_string(), self.url_depth());

        features
    }

    fn url_depth(&mut self) -> usize {
        let mut matches: Vec<regex::Match> = Vec::new();
        let mut num;

        let re = Regex::new(r"[^/]+").unwrap();
        for c in re.captures_iter(&self.url) {
            matches.push(c.get(0).unwrap())
        }

        num = matches.len();
        if uri_http_or_https(*matches.get(0).unwrap()) {
            num -= 2;
        } else {
            num -= 1;
        }
        num
    }

    fn process_and_extract(&mut self) -> HashMap<String, usize> {
        let mut features = HashMap::new();
        let f_tags: Vec<&str> = vec![
            "p",
            "ul",
            "ol",
            "dl",
            "div",
            "pre",
            "table",
            "select",
            "article",
            "section",
            "blockquote",
            "a",
            "img",
            "script",
        ];

        let mut f_checks: Vec<&str> = vec![
            "text_blocks",
            "url_depth",
            "amphtml",
            "fb_pages",
            "og_article",
            "words",
            "schema_org",
        ];

        f_checks.append(&mut f_tags.clone());
        for f in f_checks {
            features.insert(f.to_string(), 0);
        }

        let mut sink = Sink {
            next_id: 1,
            names: HashMap::new(),
            level_tracker: HashMap::new(),
            features,
        };

        let parser = parse_document(sink, Default::default());

        sink = parser
            .from_utf8()
            .read_from(&mut self.doc.as_bytes())
            .unwrap();

        sink.features
    }
}

#[derive(Debug)]
struct Sink {
    next_id: usize,
    names: HashMap<usize, QualName>,
    features: HashMap<String, usize>,
    level_tracker: HashMap<usize, usize>,
}

impl Sink {
    fn get_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 2;
        id
    }
}

impl TreeSink for Sink {
    type Handle = usize;
    type Output = Self;
    fn finish(self) -> Self {
        self
    }

    fn get_document(&mut self) -> usize {
        0
    }

    fn same_node(&self, x: &usize, y: &usize) -> bool {
        x == y
    }

    fn elem_name(&self, target: &usize) -> ExpandedName {
        let e = self.names.get(target).expect("not an element").expanded();
        e
    }

    // everytime the parser identifies a new element, our sink will figure out
    // if the element is part of the subset used by our classifier. if that is
    // the case, increases the respective feature counter.
    fn create_element(&mut self, name: QualName, attrs: Vec<Attribute>, _: ElementFlags) -> usize {
        let id = self.get_id();
        self.names.insert(id, name.clone());

        // increases count on feature map for selected tags
        let elem = name.local.to_string();
        self.features.entry(elem.clone()).and_modify(|v| *v += 1);

        // seaches for `<meta property="{og:},{fb:}..." />`
        if elem == "meta" {
            for a in attrs.clone() {
                let attr = a.value.to_string();

                if starts_with(&attr, "og:") {
                    self.features
                        .entry("og_article".to_string())
                        .and_modify(|v| *v = 1);
                }

                if starts_with(&attr, "fb:") {
                    self.features
                        .entry("fb_pages".to_string())
                        .and_modify(|v| *v = 1);
                }
            }
        }

        // checks if page is AMP compatible
        if elem == "link" {
            for a in attrs {
                if a.value.to_string() == "amphtml" {
                    self.features
                        .entry("amphtml".to_string())
                        .and_modify(|v| *v = 1);
                }
            }
        }

        id
    }

    fn create_comment(&mut self, _: StrTendril) -> usize {
        self.get_id()
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        println!("Err doc parsing: {}", msg);
    }

    // everytime the append node is required from our sink, it will either
    // 1) update the tag level tracker by setting the new node's level as +1
    //    from its parent and
    // 2) if the node to append is of type `text`, it will proceed to calculate
    //    the number of words in the text node and add that information to the
    //    feature list. It will also decide whether the text is a `text_block`
    //    and update the feature list accordingly.
    fn append(&mut self, pid: &usize, child: NodeOrText<usize>) {
        match child {
            AppendNode(n) => {
                // calculates the current node level based on its parent (or
                // lack of it)
                let level = match self.level_tracker.get(&pid) {
                    Some(pl) => pl + 1,
                    None => 1,
                };
                self.level_tracker.insert(n, level);
            }
            // calculates `words` and `text_blocks` features
            AppendText(t) => {
                let parent_level = self.level_tracker.get(pid).unwrap();
                let parent_el = self.names.get(pid).unwrap().local.to_string();

                if parent_el == "p" {
                    // words
                    let text = escape_default(&t);
                    let num_words: Vec<&str> = text.split(' ').collect();
                    self.features
                        .entry("words".to_string())
                        .and_modify(|v| *v += num_words.len());

                    // text_blocks
                    if num_words.len() > 400 && *parent_level > 1 && *parent_level < 11 {
                        self.features
                            .entry("text_blocks".to_string())
                            .and_modify(|v| *v += 1);
                    }
                }
            }
        }
    }

    fn add_attrs_if_missing(&mut self, _: &usize, _attrs: Vec<Attribute>) {}

    // unimplemented traits
    fn append_based_on_parent_node(
        &mut self,
        _element: &usize,
        _prev_element: &usize,
        _new_node: NodeOrText<usize>,
    ) {
        unimplemented!();
    }
    fn get_template_contents(&mut self, _: &usize) -> usize {
        unimplemented!();
    }
    fn set_quirks_mode(&mut self, _mode: QuirksMode) {}
    fn append_doctype_to_document(&mut self, _: StrTendril, _: StrTendril, _: StrTendril) {}
    fn remove_from_parent(&mut self, _target: &usize) {
        unimplemented!();
    }
    fn reparent_children(&mut self, _node: &usize, _new_parent: &usize) {
        unimplemented!();
    }
    fn create_pi(&mut self, _: StrTendril, _: StrTendril) -> usize {
        unimplemented!()
    }
    fn append_before_sibling(&mut self, _sibling: &usize, _new_node: NodeOrText<usize>) {
        unimplemented!();
    }
}

// helper functions
fn uri_http_or_https(m: regex::Match) -> bool {
    if m.end() == 5 || m.end() == 6 {
        true
    } else {
        false
    }
}

pub fn escape_default(s: &str) -> String {
    s.chars().flat_map(|c| c.escape_default()).collect()
}

pub fn starts_with(attr: &str, pattern: &str) -> bool {
    match Regex::new(pattern).unwrap().captures(&attr) {
        Some(_) => true,
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_url() {
        let mut url = "http://url.com".to_string();
        let mut ext1 = FeatureExtractor::from_document("".to_string(), url);
        assert_eq!(ext1.url_depth(), 0);

        url = "http://url.com/".to_string();
        let mut ext2 = FeatureExtractor::from_document("".to_string(), url);
        assert_eq!(ext2.url_depth(), 0);

        url = "http://url.com/another/path/here?test".to_string();
        let mut ext3 = FeatureExtractor::from_document("".to_string(), url);
        assert_eq!(ext3.url_depth(), 3);

        url = "www.url.com/another/path".to_string();
        let mut ext4 = FeatureExtractor::from_document("".to_string(), url);
        assert_eq!(ext4.url_depth(), 2);
    }
}
