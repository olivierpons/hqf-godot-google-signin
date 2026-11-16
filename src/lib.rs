//! Google Sign-In for Godot 4.7+ Android games.
//!
//! The pure logic (configuration, nonce, error codes, token claims, the sign-in flow and
//! the Credential Manager calls) builds and tests on any host; [`godot_api`] binds it to
//! Godot.

pub mod android;
pub mod config;
pub mod error;
pub mod flow;
pub mod godot_api;
pub mod id_token;
pub mod java;

pub use config::{Mode, SignInConfig};
pub use error::{ErrorCode, SignInError};
pub use id_token::{IdTokenClaims, TokenError};

#[allow(
    unsafe_code,
    reason = "gdext declares ExtensionLibrary as an unsafe trait; the impl has no body"
)]
mod entry {
    use godot::prelude::*;

    /// Extension entry point registered with Godot.
    struct HqfGoogleSignInExtension;

    #[gdextension(entry_symbol = hqf_google_signin_init)]
    unsafe impl ExtensionLibrary for HqfGoogleSignInExtension {}
}
