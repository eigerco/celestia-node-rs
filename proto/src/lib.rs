#![allow(clippy::all)]
#![allow(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod serializers;

include!(concat!(env!("OUT_DIR"), "/mod.rs"));
