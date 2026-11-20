//! "Masuk dengan Google": OpenID Connect authorization-code flow with PKCE, state and nonce.
//! The browser only ever carries two random httpOnly cookies; the code, the tokens and the
//! client secret stay on the server.

pub mod client;
pub mod cookies;
pub mod flow;
pub mod id_token;

pub use client::{GoogleClient, GoogleError};
