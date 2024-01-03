#![doc = include_str!("../../README.md")]
#![cfg(not(target_arch = "wasm32"))]

mod common;
mod native;
mod server;

pub use common::run;
