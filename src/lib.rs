#![allow(dead_code)]
#![forbid(unsafe_code)]

extern crate url;
extern crate html5ever;

pub mod classifier;
pub mod predictor;

pub mod feature_extractor;
pub mod feature_extractor_tokenizer;

#[cfg(test)]
#[macro_use]
extern crate matches;
