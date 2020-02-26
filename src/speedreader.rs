use lol_html::OutputSink;
use lol_html::Selector;
use std::any::Any;
use thiserror::Error;
use url::Url;

use super::rewriter_config_builder::*;
use super::speedreader_heuristics::SpeedReaderHeuristics;
use super::speedreader_streaming::SpeedReaderStreaming;
use super::whitelist::Whitelist;

#[derive(Error, Debug, PartialEq)]
pub enum SpeedReaderError {
    #[error("Invalid article URL.")]
    InvalidUrl(String),
    #[error("Document parsing error: `{0}`")]
    DocumentParseError(String),
    #[error("Document rewriting error: `{0}`")]
    RewritingError(String),
    #[error("Configuration error: `{0}`")]
    ConfigurationError(String),
}

impl From<lol_html::errors::RewritingError> for SpeedReaderError {
    fn from(err: lol_html::errors::RewritingError) -> Self {
        SpeedReaderError::RewritingError(err.to_string())
    }
}

impl From<lol_html::errors::EncodingError> for SpeedReaderError {
    fn from(err: lol_html::errors::EncodingError) -> Self {
        SpeedReaderError::RewritingError(err.to_string())
    }
}

impl From<url::ParseError> for SpeedReaderError {
    fn from(err: url::ParseError) -> Self {
        SpeedReaderError::InvalidUrl(err.to_string())
    }
}

impl From<std::io::Error> for SpeedReaderError {
    fn from(err: std::io::Error) -> Self {
        SpeedReaderError::DocumentParseError(err.to_string())
    }
}

pub trait SpeedReaderProcessor {
    fn write(&mut self, input: &[u8]) -> Result<(), SpeedReaderError>;
    fn end(&mut self) -> Result<(), SpeedReaderError>;
}

#[derive(Clone, Debug)]
pub struct SpeedReaderConfig {
    pub domain: String,
    pub url_rules: Vec<String>,
    pub declarative_rewrite: Option<RewriteRules>,
}

#[derive(Clone, Debug)]
pub struct AttributeRewrite {
    pub selector: String,
    pub attribute: String,
    pub to_attribute: String,
    pub element_name: String,
}

#[derive(Clone, Debug)]
pub struct RewriteRules {
    pub main_content: Vec<String>,
    pub main_content_cleanup: Vec<String>,
    pub delazify: bool,
    pub fix_embeds: bool,
    pub content_script: Option<String>,
    pub preprocess: Vec<AttributeRewrite>,
}

impl RewriteRules {
    pub fn get_content_handlers(&self, url: &Url) -> Vec<(Selector, ContentFunction)> {
        rewrite_rules_to_content_handlers(self, &url.origin().ascii_serialization())
    }
}

impl RewriteRules {
    pub fn get_main_content_selectors(&self) -> Vec<&str> {
        self.main_content.iter().map(AsRef::as_ref).collect()
    }
    pub fn get_content_cleanup_selectors(&self) -> Vec<&str> {
        self.main_content_cleanup
            .iter()
            .map(AsRef::as_ref)
            .collect()
    }
}

pub struct SpeedReader {
    whitelist: Whitelist,
    url_engine: adblock::engine::Engine,
}

impl SpeedReader {
    pub fn new() -> Self {
        let mut whitelist = Whitelist::default();
        whitelist.load_predefined();
        let url_engine = adblock::engine::Engine::from_rules(&whitelist.get_url_rules());
        SpeedReader {
            whitelist,
            url_engine,
        }
    }

    pub fn with_whitelist(whitelist: Whitelist) -> Self {
        let url_engine = adblock::engine::Engine::from_rules(&whitelist.get_url_rules());
        SpeedReader {
            whitelist,
            url_engine,
        }
    }

    pub fn url_readable(&self, url: &str) -> Option<bool> {
        let matched = self.url_engine.check_network_urls(url, url, "document");
        if matched.exception.is_some() {
            Some(false)
        } else if matched.matched {
            Some(true)
        } else {
            None
        }
    }

    pub fn find_config(&self, article_url: &str) -> (Option<&SpeedReaderConfig>, Box<dyn Any>) {
        let url = Url::parse(article_url).unwrap();
        let config = self
            .whitelist
            .get_configuration(&url.domain().unwrap_or_default().replace("www.", ""));

        let content_handlers;
        match config {
            Some(SpeedReaderConfig {
                domain: _,
                url_rules: _,
                declarative_rewrite: Some(rewrite),
            }) => content_handlers = rewrite.get_content_handlers(&url),
            _ => content_handlers = vec![],
        }

        (config, Box::new(content_handlers))
    }

    pub fn get_rewriter<'h, O: OutputSink + 'h>(
        &'h self,
        article_url: &str,
        config: Option<&SpeedReaderConfig>,
        extra: &'h Box<dyn Any>,
        output_sink: O,
    ) -> Result<Box<dyn SpeedReaderProcessor + 'h>, SpeedReaderError> {
        let url = Url::parse(article_url).unwrap();
        // let extra = extra as &dyn Any;
        if let Some(content_handlers) = extra.downcast_ref::<Vec<(Selector, ContentFunction)>>() {
            match config {
                Some(_) => Ok(Box::new(SpeedReaderStreaming::try_new(
                    url,
                    output_sink,
                    content_handlers,
                )?)),
                _ => Ok(Box::new(SpeedReaderHeuristics::try_new(
                    url.as_str(),
                    output_sink,
                )?)),
            }
        } else {
            Err(SpeedReaderError::ConfigurationError(
                "The configuration `extra` parameter could not be unmarshalled to expected type"
                    .to_owned(),
            ))
        }
    }
}
