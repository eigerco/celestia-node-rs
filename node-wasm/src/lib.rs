#![doc = include_str!("../README.md")]
#![cfg(target_arch = "wasm32")]

pub mod client;
mod commands;
pub mod error;
mod ports;
pub mod utils;
mod worker;
mod wrapper;

pub use celestia_grpc::TxClient;

#[cfg(test)]
wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);
