#![allow(dead_code)]
#![forbid(unsafe_code)]
extern crate html5ever;
extern crate lol_html;
extern crate url;
extern crate adblock;

#[cfg(test)]
#[macro_use]
extern crate matches;

pub mod classifier;
pub mod speedreader;
mod rewriter_config_builder;
mod speedreader_heuristics;
mod speedreader_streaming;

pub mod whitelist;

pub use self::speedreader::{
    AttributeRewrite, RewriteRules, SpeedReader, SpeedReaderConfig, SpeedReaderError,
    SpeedReaderProcessor,
};
