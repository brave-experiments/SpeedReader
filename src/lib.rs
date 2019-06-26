#![allow(dead_code)]
#![forbid(unsafe_code)]
extern crate url;
extern crate html5ever;

pub mod classifier;

#[cfg(test)]
#[macro_use]
extern crate matches;

use readability;
use std::io::Read;
