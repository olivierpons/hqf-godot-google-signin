//! What a sign-in asks Google for.

use crate::error::{ErrorCode, SignInError};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;

/// Suffix of every OAuth client ID issued by Google Cloud.
pub const CLIENT_ID_SUFFIX: &str = ".apps.googleusercontent.com";

/// Bytes of randomness in a generated nonce.
pub const NONCE_BYTES: usize = 32;

/// How the account sheet is offered to the player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[non_exhaustive]
pub enum Mode {
    /// Accounts that already signed in to this app first, signed in at once when there is
    /// only one; every Google account of the device when none did.
    #[default]
    Automatic,
    /// Every Google account of the device, the player picks one.
    AccountChooser,
    /// The "Sign in with Google" flow meant to follow a press on a sign-in button.
    Button,
    /// Accounts that already signed in to this app only, without any sheet when there is
    /// only one; fails with [`ErrorCode::NoCredential`] otherwise.
    Silent,
}

/// Settings of a sign-in request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignInConfig {
    web_client_id: String,
    hosted_domain: Option<String>,
    request_verified_phone_number: bool,
}

impl SignInConfig {
    /// Settings for the given Web application client ID of Google Cloud.
    ///
    /// # Errors
    ///
    /// [`ErrorCode::InvalidConfig`] when the ID is not a Google OAuth client ID: it must
    /// end with [`CLIENT_ID_SUFFIX`], have something before it and hold no whitespace.
    ///
    /// ```
    /// use hqf_godot_google_signin::SignInConfig;
    /// let id = "123-abc.apps.googleusercontent.com";
    /// assert_eq!(SignInConfig::new(id).unwrap().web_client_id(), id);
    /// assert!(SignInConfig::new("123").is_err());
    /// ```
    pub fn new(web_client_id: &str) -> Result<Self, SignInError> {
        let id = web_client_id.trim();
        let prefix = id.strip_suffix(CLIENT_ID_SUFFIX).unwrap_or_default();
        if prefix.is_empty() || id.chars().any(char::is_whitespace) {
            return Err(SignInError::new(
                ErrorCode::InvalidConfig,
                format!("web client ID must look like <id>{CLIENT_ID_SUFFIX}"),
            ));
        }
        Ok(Self {
            web_client_id: id.to_owned(),
            hosted_domain: None,
            request_verified_phone_number: false,
        })
    }

    /// Only offers accounts of this Google Workspace domain; empty offers every account.
    #[must_use]
    pub fn with_hosted_domain(mut self, domain: &str) -> Self {
        let domain = domain.trim();
        self.hosted_domain = (!domain.is_empty()).then(|| domain.to_owned());
        self
    }

    /// Asks Google for a verified phone number in the ID token.
    #[must_use]
    pub const fn with_verified_phone_number(mut self, wanted: bool) -> Self {
        self.request_verified_phone_number = wanted;
        self
    }

    /// The Web application client ID the ID token is issued for.
    pub fn web_client_id(&self) -> &str {
        &self.web_client_id
    }

    /// The Google Workspace domain accounts are filtered on, if any.
    pub fn hosted_domain(&self) -> Option<&str> {
        self.hosted_domain.as_deref()
    }

    /// Whether a verified phone number is asked for.
    pub const fn request_verified_phone_number(&self) -> bool {
        self.request_verified_phone_number
    }
}

/// A nonce to bind an ID token to one sign-in, given by the game or generated.
///
/// # Errors
///
/// [`ErrorCode::InvalidConfig`] when [`given`](nonce) holds whitespace, and
/// [`ErrorCode::Bridge`] when the system random source fails.
///
/// ```
/// use hqf_godot_google_signin::config::nonce;
/// assert_eq!(nonce("abc").unwrap(), "abc");
/// assert_eq!(nonce("").unwrap().len(), 43);
/// ```
pub fn nonce(given: &str) -> Result<String, SignInError> {
    if given.chars().any(char::is_whitespace) {
        return Err(SignInError::new(
            ErrorCode::InvalidConfig,
            "nonce must not hold whitespace",
        ));
    }
    if !given.is_empty() {
        return Ok(given.to_owned());
    }
    let mut bytes = [0_u8; NONCE_BYTES];
    getrandom::fill(&mut bytes)
        .map_err(|error| SignInError::new(ErrorCode::Bridge, error.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const ID: &str = "1234-abcd.apps.googleusercontent.com";

    #[test]
    fn accepts_a_client_id_and_trims_it() {
        let config = SignInConfig::new(&format!("  {ID}\n")).unwrap();
        assert_eq!(config.web_client_id(), ID);
        assert_eq!(config.hosted_domain(), None);
        assert!(!config.request_verified_phone_number());
    }

    #[test]
    fn refuses_bad_client_ids() {
        for bad in [
            "",
            "1234",
            CLIENT_ID_SUFFIX,
            "12 34.apps.googleusercontent.com",
            "1234.apps.googleusercontent.com.evil",
        ] {
            let error = SignInConfig::new(bad).unwrap_err();
            assert_eq!(error.code, ErrorCode::InvalidConfig, "{bad:?}");
            assert!(error.message.contains(CLIENT_ID_SUFFIX));
        }
    }

    #[test]
    fn hosted_domain_and_phone() {
        let config = SignInConfig::new(ID)
            .unwrap()
            .with_hosted_domain(" example.com ")
            .with_verified_phone_number(true);
        assert_eq!(config.hosted_domain(), Some("example.com"));
        assert!(config.request_verified_phone_number());
        let config = config
            .with_hosted_domain("  ")
            .with_verified_phone_number(false);
        assert_eq!(config.hosted_domain(), None);
        assert!(!config.request_verified_phone_number());
    }

    #[test]
    fn default_mode_is_automatic() {
        assert_eq!(Mode::default(), Mode::Automatic);
    }

    #[test]
    fn given_nonce_is_kept() {
        assert_eq!(nonce("n-0_1").unwrap(), "n-0_1");
    }

    #[test]
    fn nonce_with_whitespace_is_refused() {
        assert_eq!(nonce("a b").unwrap_err().code, ErrorCode::InvalidConfig);
        assert_eq!(nonce("\t").unwrap_err().code, ErrorCode::InvalidConfig);
    }

    #[test]
    fn generated_nonces_are_url_safe_and_differ() {
        let first = nonce("").unwrap();
        let second = nonce("").unwrap();
        assert_ne!(first, second);
        assert_eq!(URL_SAFE_NO_PAD.decode(&first).unwrap().len(), NONCE_BYTES);
        assert!(
            first
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        );
    }

    proptest! {
        #[test]
        fn any_clean_prefix_is_accepted(prefix in "[0-9a-z-]{1,40}") {
            let id = format!("{prefix}{CLIENT_ID_SUFFIX}");
            let config = SignInConfig::new(&id).unwrap();
            prop_assert_eq!(config.web_client_id(), id.as_str());
        }

        #[test]
        fn nonce_without_whitespace_round_trips(given in "[!-~]{1,64}") {
            prop_assert_eq!(nonce(&given).unwrap(), given);
        }
    }
}
