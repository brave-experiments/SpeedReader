#![allow(dead_code)]
#![forbid(unsafe_code)]
extern crate url;
extern crate html5ever;
extern crate lol_html;

#[cfg(test)]
#[macro_use]
extern crate matches;

pub mod classifier;
pub mod speedreader;
pub mod streaming;
