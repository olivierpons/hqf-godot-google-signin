//! Google Sign-In for Godot 4.7+ Android games.
//!
//! The pure logic (configuration, nonce, error codes) lives in plain Rust modules that
//! build and test on any host.

pub mod config;
pub mod error;
pub mod flow;
pub mod id_token;

pub use config::{Mode, SignInConfig};
pub use error::{ErrorCode, SignInError};
pub use id_token::{IdTokenClaims, TokenError};
