use std::env;

use anyhow::Result;
use celestia_types::nmt::Namespace;
use jsonrpsee::http_client::HttpClient;
use rand::RngCore;

const CONN_STR: &str = "http://localhost:26658";

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AuthLevel {
    Public,
    Read,
    Write,
    Admin,
}

fn token_from_env(auth_level: AuthLevel) -> Result<Option<String>> {
    match auth_level {
        AuthLevel::Public => Ok(None),
        AuthLevel::Read => Ok(Some(env::var("CELESTIA_NODE_AUTH_TOKEN_READ")?)),
        AuthLevel::Write => Ok(Some(env::var("CELESTIA_NODE_AUTH_TOKEN_WRITE")?)),
        AuthLevel::Admin => Ok(Some(env::var("CELESTIA_NODE_AUTH_TOKEN_ADMIN")?)),
    }
}

pub fn test_client(auth_level: AuthLevel) -> Result<HttpClient> {
    let _ = dotenvy::dotenv();
    let token = token_from_env(auth_level)?;
    Ok(celestia_rpc::client::new_http(CONN_STR, token.as_deref())?)
}

pub fn random_ns() -> Namespace {
    Namespace::const_v0(random_bytes_array())
}

pub fn random_bytes(length: usize) -> Vec<u8> {
    let mut bytes = vec![0; length];
    rand::thread_rng().fill_bytes(&mut bytes);
    bytes
}

pub fn random_bytes_array<const N: usize>() -> [u8; N] {
    std::array::from_fn(|_| rand::random())
}
