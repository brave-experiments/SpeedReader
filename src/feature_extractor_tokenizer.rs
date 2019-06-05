use html5ever::tendril::Tendril;
use html5ever::tokenizer::{BufferQueue, EndTag, StartTag, Token, TokenSink, TokenSinkResult};
use html5ever::tokenizer::{
    CharacterTokens, CommentToken, DoctypeToken, EOFToken, NullCharacterToken, ParseError, Tag,
    TagToken, Tokenizer, TokenizerOpts,
};

use regex::Regex;

use std::collections::HashMap;
use std::default::Default;
use std::str::FromStr;
use std::string::String;

pub struct FeatureExtractorT {
    doc: String,
    url: String,
}

// all the feature extraction functions here
impl FeatureExtractorT {
    pub fn from_document(doc: String, url: String) -> FeatureExtractorT {
        FeatureExtractorT { doc, url }
    }

    pub fn extract(&mut self) -> HashMap<String, usize> {
        let mut features = HashMap::new();
        let feature_tags: Vec<String> = vec![
            String::from("p"),
            String::from("ul"),
            String::from("ol"),
            String::from("dl"),
            String::from("div"),
            String::from("pre"),
            String::from("table"),
            String::from("select"),
            String::from("article"),
            String::from("section"),
            String::from("blockquote"),
            String::from("a"),
            String::from("img"),
            String::from("script"),
            String::from("text_blocks"),
            String::from("url_depth"),
            String::from("amphtml"),
            String::from("fb_pages"),
            String::from("og_article"),
            String::from("words"),
            String::from("schema_org"),
        ];

        for f in feature_tags {
            features.insert(f.to_string(), 0);
        }

        let sink = SinkFeatureExtractor {
            is_char_run: false,
            is_within_p_tag: false,
            features,
            level: 0,
            current_word: Vec::new(),
        };

        let data = Tendril::from_str(&self.doc).unwrap();
        let mut input = BufferQueue::new();
        input.push_back(data);

        let mut tokenizer = Tokenizer::new(
            sink,
            TokenizerOpts {
                ..Default::default()
            },
        );

        // runs tokenizer with `input` data
        let _ = tokenizer.feed(&mut input);
        assert!(input.is_empty());
        tokenizer.end();

        let mut features = tokenizer.sink.features;
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
}

#[derive(Clone)]
struct SinkFeatureExtractor {
    is_char_run: bool,
    is_within_p_tag: bool,
    features: HashMap<String, usize>,
    level: usize,
    current_word: Vec<char>,
}

impl SinkFeatureExtractor {
    fn is_char(&mut self, is_char: bool) {
        match (self.is_char_run, is_char) {
            // finishes processing word
            (true, false) => {
                let s: String = self.current_word.clone().into_iter().collect();
                let num_words: Vec<&str> = s.split(' ').collect();

                // adds num_words to features
                self.features
                    .entry("words".to_string())
                    .and_modify(|v| *v += num_words.len());

                // checks if current set of words is `text_block`
                // #TODO refactor
                // #TODO params to consts/configs
                if self.level > 1
                    && self.level < 11
                    && self.is_within_p_tag
                    && num_words.len() > 400
                {
                    self.features
                        .entry("text_blocks".to_string())
                        .and_modify(|v| *v += 1);
                }

                self.current_word.clear();
            }
            _ => (),
        }
        self.is_char_run = is_char;
    }

    fn do_char(&mut self, c: char) {
        self.is_char(true);
        if c.is_alphanumeric() || c == ' ' {
            self.current_word.push(c);
        }
    }

    fn update_level(&mut self, tag: Tag) -> Tag {
        match tag.kind {
            StartTag => {
                if !tag.self_closing {
                    self.level += 1
                }
            }
            EndTag => self.level -= 1,
        };
        tag
    }
}

// implement TokenSink for TokenProcessor
impl TokenSink for SinkFeatureExtractor {
    type Handle = ();

    fn process_token(&mut self, token: Token, _line_num: u64) -> TokenSinkResult<()> {
        match token {
            TagToken(tag) => {
                let el_name = tag.name.to_string();
                self.is_char(false);

                //tag = self.update_level(tag);
                match tag.kind {
                    StartTag => {
                        // updates tag level
                        if !tag.self_closing {
                            self.level += 1;
                        }

                        // marks whether current tag is being processed within
                        // a text block (<p> tag) #TODO: not accounting for
                        // possible nested text blocks (needs a level count
                        // instead of bool)
                        if tag.name == "p".to_string() {
                            self.is_within_p_tag = true;
                        }

                        // increases count of selected features
                        self.features.entry(el_name.clone()).and_modify(|v| *v += 1);
                    }

                    EndTag => {
                        // updates tag level
                        self.level -= 1;

                        if tag.name == "p".to_string() {
                            self.is_within_p_tag = false;
                        }
                    }
                };

                // checks page compatibilities #TODO: refactor
                let attrs = tag.attrs;

                if el_name == "meta".to_string() {
                    for a in attrs.clone() {
                        let attr = a.value.to_string();

                        // compatible with open graph?
                        match Regex::new(r"og:").unwrap().captures(&attr) {
                            Some(_) => {
                                self.features
                                    .entry("og_article".to_string())
                                    .and_modify(|v| *v = 1);
                            }
                            None => (),
                        };

                        // compatible with facebook pages?
                        match Regex::new(r"fb:").unwrap().captures(&attr) {
                            Some(_) => {
                                self.features
                                    .entry("fb_pages".to_string())
                                    .and_modify(|v| *v = 1);
                            }
                            None => (),
                        }
                    }
                }

                // AMP compatible?
                if el_name == "link".to_string() {
                    for a in attrs {
                        if a.value.to_string() == "amphtml" {
                            self.features
                                .entry("amphtml".to_string())
                                .and_modify(|v| *v = 1);
                        }
                    }
                }
            }

            CharacterTokens(b) => {
                for c in b.chars() {
                    self.do_char(c);
                }
            }

            NullCharacterToken => self.do_char('\0'),
            DoctypeToken(_) => (),
            CommentToken(_) => (),
            ParseError(_) => (),
            EOFToken => (),
        }

        TokenSinkResult::Continue
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_depth_url() {
        let mut url = "http://url.com".to_string();
        let mut ext1 = FeatureExtractorT::from_document("".to_string(), url);
        assert_eq!(ext1.url_depth(), 0);

        url = "http://url.com/".to_string();
        let mut ext2 = FeatureExtractorT::from_document("".to_string(), url);
        assert_eq!(ext2.url_depth(), 0);

        url = "http://url.com/another/path/here?test".to_string();
        let mut ext3 = FeatureExtractorT::from_document("".to_string(), url);
        assert_eq!(ext3.url_depth(), 3);

        url = "www.url.com/another/path".to_string();
        let mut ext4 = FeatureExtractorT::from_document("".to_string(), url);
        assert_eq!(ext4.url_depth(), 2);
    }
}
