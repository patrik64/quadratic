//! Quadratic File Service
//!
//! A file servic for that consumes transactions from a queue, applies them to
//! a grid and writes them to S3.

mod auth;
mod config;
mod error;
mod file;
mod health;
mod server;
mod state;
mod storage;
#[cfg(test)]
mod test_util;
mod truncate;

use error::Result;

#[tokio::main]
async fn main() -> Result<()> {
    server::serve().await
}
