//! Cloudflare WARP client, re-implementing the functionality of wgcf
//! (https://github.com/ViRb3/wgcf) in Rust so waX VPN can manage WARP
//! registration natively.
//!
//! This module is strictly opt-in: nothing here runs unless the user
//! explicitly asks to register. waX VPN is not affiliated with Cloudflare.

pub mod client;
pub mod model;
pub mod store;

pub use client::WarpClient;
pub use model::*;
pub use store::{delete_account, load_account, save_account, WarpAccount};

#[derive(Debug, thiserror::Error)]
pub enum WarpError {
    #[error("http error: {0}")]
    Http(String),
    #[error("WARP API error {code}: {message}")]
    Api { code: i64, message: String },
    #[error("unexpected response: {0}")]
    BadResponse(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Core(#[from] wax_core::CoreError),
    #[error("no WARP registration found; register first")]
    NotRegistered,
}
