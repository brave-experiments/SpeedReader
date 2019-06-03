use std::borrow::Cow;
use std::collections::HashMap;
use std::default::Default;
use std::string::String;
use std::vec::Vec;

use html5ever::parse_document;
use html5ever::tendril::*;
use html5ever::tree_builder::{ElementFlags, NodeOrText, QuirksMode, TreeSink};
use html5ever::{Attribute, ExpandedName, QualName};

struct Sink {
    next_id: usize,
    names: HashMap<usize, QualName>,
    features: HashMap<std::string::String, usize>,
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

    fn create_element(&mut self, name: QualName, _: Vec<Attribute>, _: ElementFlags) -> usize {
        let id = self.get_id();
        self.names.insert(id, name.clone());

        // increases count on feature map for selected tags
        let tags: Vec<&str> = vec![
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

        let el = name.local.to_string();
        if tags.iter().any(|x| String::from(*x) == el) {
            let counter = self.features.entry(el).or_insert(0);
            *counter += 1;
        }

        id
    }

    fn create_comment(&mut self, _: StrTendril) -> usize {
        self.get_id()
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {
        panic!("{}", msg);
    }

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
    fn append(&mut self, _parent: &usize, _child: NodeOrText<usize>) {}

    fn append_doctype_to_document(&mut self, _: StrTendril, _: StrTendril, _: StrTendril) {}
    fn add_attrs_if_missing(&mut self, _: &usize, _attrs: Vec<Attribute>) {
        unimplemented!();
    }
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

pub struct FeatureExtractor {
    doc: String,
}

impl FeatureExtractor {
    pub fn from_document(d: String) -> FeatureExtractor {
        FeatureExtractor { doc: d }
    }

    pub fn extract(&mut self) -> HashMap<String, usize> {
        let mut sink = Sink {
            next_id: 1,
            names: HashMap::new(),
            features: HashMap::new(),
        };

        let parser = parse_document(sink, Default::default());

        sink = parser
            .from_utf8()
            .read_from(&mut self.doc.as_bytes())
            .unwrap();

        sink.features
    }
}
