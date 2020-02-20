#![allow(dead_code)]
#![forbid(unsafe_code)]
extern crate url;
extern crate html5ever;
extern crate lol_html;

#[cfg(test)]
#[macro_use]
extern crate matches;

mod speedreader_heuristics;
pub mod speedreader_streaming;
pub mod classifier;
pub mod speedreader;
pub mod streaming;
pub mod whitelist;