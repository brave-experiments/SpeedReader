use url::Url;
pub use super::whitelist::Whitelist;
pub use super::streaming::rewriter_config_builder::{RewriterConfigBuilder, content_handlers};
use super::speedreader_streaming::SpeedReaderStreaming;
use super::speedreader_heuristics::SpeedReaderHeuristics;

#[derive(Debug, PartialEq)]
pub enum SpeedReaderError {
    InvalidUrl(String),
    DocumentParseError(String),
    RewritingError(String)
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
    pub declarative_rewrite: Option<RewriteRules>
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

pub fn blocking_content_rewrite(article_url: &str, content: &[u8]) -> Result<Vec<u8>, SpeedReaderError> {
    let url = Url::parse(article_url).unwrap();

    let mut whitelist = Whitelist::default();
    whitelist.load_predefined();
    let maybe_config = whitelist.get_configuration(&url.domain().unwrap_or_default().replace("www.", ""));

    let mut buf = vec![];
    match maybe_config {
        Some(SpeedReaderConfig {domain: _, url_rules: _, declarative_rewrite: Some(rewrite)}) => {
            let r_config = RewriterConfigBuilder::new(
                rewrite,
                &url.origin().ascii_serialization(),
            );
    
            let mut rewriter = SpeedReaderStreaming::try_new(
                url,
                |c: &[u8]| {
                    buf.extend_from_slice(c);
                },
                &r_config
            )?;
    
            rewriter.write(content)?;
            rewriter.end()?;
        },
        _ => {
            let mut rewriter = SpeedReaderHeuristics::try_new(
                url.as_str(),
                |c: &[u8]| {
                    buf.extend_from_slice(c)
                }
            )?;
    
            rewriter.write(content)?;
            rewriter.end()?;
        }
    }
    
    Ok(buf)
}
