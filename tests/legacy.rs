#![allow(dead_code)]
extern crate url;
extern crate readability;
extern crate speedreader;
extern crate html5ever;
extern crate distance;
extern crate regex;

#[macro_use]
extern crate lazy_static;


use readability::extractor;
use speedreader::classifier::feature_extractor::FeatureExtractor;
use std::fs::File;
use std::io::Read;
use url::Url;

use std::rc::Rc;
use std::vec::Vec;
use html5ever::rcdom::{Node, Handle};
use html5ever::rcdom::NodeData::{Element, Text};
use regex::Regex;

static SAMPLES_PATH: &'static str = "./tests/samples/";

fn load_test_files(test_name: &str) -> String {
    let mut expected = "".to_owned();
    let mut exp_f = File::open(format!("{}/{}/expected.html", SAMPLES_PATH, test_name)).unwrap();
    exp_f.read_to_string(&mut expected).unwrap();

    expected.to_owned()
}

pub fn extract_flattened_tree(handle: Handle, tags_attrs: &Vec<(&str, &str)>, 
                               flattened_nodes: &mut Vec<Rc<Node>>) -> Vec<Rc<Node>> {
    for child in handle.children.borrow().iter() {
        let c = child.clone();
        match c.data {
            Text { .. } => {
                flattened_nodes.push(c.clone());
            },
            Element { ref name, ref attrs, .. } => {
                let t = name.local.as_ref();
                for a in attrs.borrow().iter() {
                    let t = t.to_lowercase();
                    let a = a.value.to_string().to_lowercase();

                    // check if current node name and attr match expected
                    for (tag_name, attr_name) in tags_attrs.iter() {
                        // let (tag_name, attr_name): (&str, &str) = ta;
                        if &t == *tag_name && &a == *attr_name {
                            flattened_nodes.push(c.clone());
                        }
                    }
                }
                // if type Element, traverse to children in next iteration
                extract_flattened_tree(child.clone(), tags_attrs, flattened_nodes);
            },
            _ => (),
        }
    }
    flattened_nodes.to_vec()
}

pub fn extract_text(handle: &Handle) -> String {
    match handle.data {
        Text { ref contents } => {
            contents.borrow().trim().to_owned()
        },
        _ => {
            handle.children.borrow().iter().map(|c| extract_text(c)).collect::<Vec<_>>().join(" ")
        }
    }
}


fn test_contents(name: &str) {
    let url = Url::parse("http://url.com").unwrap();
    let mut source_f =
        File::open(format!("{}/{}/source.html", SAMPLES_PATH, name))
        .unwrap();

    // opens and parses the expected final result into a rcdom 
    // (for comparing with the result)
    let expected_string = load_test_files(name);
    let expected = FeatureExtractor::parse_document(
        &mut expected_string.as_bytes(), &url.to_string()
    );

    // checks full flattened tree for a subset of (tags, attrs)
    let mut tags_attrs: Vec<(&str, &str)> = Vec::new();
    tags_attrs.push(("a", "href"));
    tags_attrs.push(("img", "src"));
    let mut expected_nodes = Vec::new();
    extract_flattened_tree(expected.dom.document.clone(), &tags_attrs, &mut expected_nodes);

    // uses the mapper build the mapper based on the source HTML
    // document
    let product = extractor::extract(&mut source_f, &url).unwrap();
    let result = FeatureExtractor::parse_document(
        &mut product.content.as_bytes(), &url.to_string()
    );

    let mut got_nodes = Vec::new();
    extract_flattened_tree(result.dom.document.clone(), &tags_attrs, &mut got_nodes);

    lazy_static! {
        static ref WHITESPACE: Regex = Regex::new(r"(\s\s+)").unwrap();
        static ref NEWLINE_ESCAPED: Regex = Regex::new(r"(\\n)").unwrap();
    }

    let expected_nodes_str: Vec<_> = expected_nodes.iter()
        .map(|n| {
            extract_text(n)
        })
        .map(|t| {
            let repl = NEWLINE_ESCAPED.replace_all(&t, " ");
            let repl = WHITESPACE.replace_all(&repl, " ");
            format!("{}", repl)
        })
        .filter(|t| !t.is_empty())
        .collect();
    let got_nodes_str: Vec<_> = got_nodes.iter()
        .map(|n| {
            extract_text(n)
        })
        .map(|t| {
            let repl = NEWLINE_ESCAPED.replace_all(&t, " ");
            let repl = WHITESPACE.replace_all(&repl, " ");
            format!("{}", repl)
        })
        .filter(|t| !t.is_empty())
        .collect();

    assert_eq!(expected_nodes_str, got_nodes_str);
}

macro_rules! test_str {
    ($name:ident) => {
        #[test]
        fn $name() {
            test_contents(stringify!($name))
        }
    }
}

test_str!(ars_1);
test_str!(cnet);
test_str!(folha);
test_str!(liberation_1);
test_str!(metadata_content_missing);
test_str!(msn);
test_str!(rtl_1);
test_str!(rtl_2);
test_str!(rtl_3);
test_str!(rtl_4);
test_str!(title_and_h1_discrepancy);
test_str!(tumblr);
test_str!(yahoo_4);
test_str!(videos_2);
test_str!(wordpress);
test_str!(pixnet);

test_str!(aclu);
test_str!(base_url);
test_str!(base_url_base_element);
test_str!(base_url_base_element_relative);
test_str!(basic_tags_cleaning);
test_str!(bbc_1);
test_str!(blogger);
test_str!(breitbart);
test_str!(bug_1255978);
test_str!(buzzfeed_1);
test_str!(citylab_1);
test_str!(clean_links);
test_str!(cnet_svg_classes);
test_str!(cnn);
test_str!(comment_inside_script_parsing);
test_str!(daringfireball_1);
test_str!(ehow_1);
test_str!(ehow_2);
test_str!(embedded_videos);
test_str!(engadget);
test_str!(gmw);
test_str!(guardian_1);
test_str!(heise);
test_str!(herald_sun_1);
test_str!(hidden_nodes);
test_str!(hukumusume);
test_str!(iab_1);
test_str!(ietf_1);
test_str!(keep_images);
test_str!(keep_tabular_data);
test_str!(la_nacion);
test_str!(lemonde_1);
test_str!(lifehacker_post_comment_load);
test_str!(lifehacker_working);
test_str!(links_in_tables);
test_str!(lwn_1);
test_str!(medicalnewstoday);
test_str!(medium_1);
test_str!(medium_3);
test_str!(mercurial);
test_str!(missing_paragraphs);
test_str!(mozilla_1);
test_str!(mozilla_2);
test_str!(normalize_spaces);
test_str!(nytimes_1);
test_str!(nytimes_2);
test_str!(nytimes_3);
test_str!(nytimes_4);
test_str!(qq);
test_str!(remove_extra_brs);
test_str!(remove_extra_paragraphs);
test_str!(remove_script_tags);
test_str!(reordering_paragraphs);
test_str!(replace_brs);
test_str!(replace_font_tags);
test_str!(salon_1);
test_str!(seattletimes_1);
test_str!(simplyfound_3);
test_str!(social_buttons);
test_str!(style_tags_removal);
test_str!(svg_parsing);
test_str!(table_style_attributes);
test_str!(telegraph);
test_str!(tmz_1);
test_str!(videos_1);
test_str!(wapo_1);
test_str!(wapo_2);
test_str!(webmd_1);
test_str!(webmd_2);
test_str!(wikipedia);
test_str!(yahoo_1);
test_str!(yahoo_2);
test_str!(yahoo_3);
test_str!(youth);
