//! Reading the claims of a Google ID token on the device.
//!
//! The signature is NOT checked here: the claims are only for showing the player who
//! signed in. A server must verify the token (signature, `aud`, `iss`, `exp`, `nonce`)
//! before trusting any of it.

use crate::error::{ErrorCode, SignInError};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::Deserialize;

/// Why an ID token could not be read.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum TokenError {
    /// The token is not three dot-separated parts.
    #[error("an ID token has 3 dot-separated parts, this one has {0}")]
    Parts(usize),
    /// The payload part is not base64url.
    #[error("payload is not base64url")]
    Base64,
    /// The payload is not the JSON object of a Google ID token.
    #[error("payload is not a Google ID token: {0}")]
    Json(String),
}

impl From<TokenError> for SignInError {
    fn from(error: TokenError) -> Self {
        Self::new(ErrorCode::InvalidToken, error.to_string())
    }
}

/// The claims of a Google ID token that a game may show or check.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct IdTokenClaims {
    /// Who issued the token: `https://accounts.google.com` or `accounts.google.com`.
    pub iss: String,
    /// The client IDs the token is meant for.
    #[serde(deserialize_with = "one_or_many")]
    pub aud: Vec<String>,
    /// The Google account's stable unique ID.
    pub sub: String,
    /// Issue time, in seconds since the Unix epoch.
    pub iat: i64,
    /// Expiry time, in seconds since the Unix epoch.
    pub exp: i64,
    /// The account's email address.
    #[serde(default)]
    pub email: Option<String>,
    /// Whether Google verified [`email`](Self::email).
    #[serde(default, deserialize_with = "bool_or_text")]
    pub email_verified: Option<bool>,
    /// Full name.
    #[serde(default)]
    pub name: Option<String>,
    /// First name.
    #[serde(default)]
    pub given_name: Option<String>,
    /// Last name.
    #[serde(default)]
    pub family_name: Option<String>,
    /// URL of the profile picture.
    #[serde(default)]
    pub picture: Option<String>,
    /// Google Workspace domain of the account.
    #[serde(default)]
    pub hd: Option<String>,
    /// The nonce the sign-in request carried.
    #[serde(default)]
    pub nonce: Option<String>,
}

impl IdTokenClaims {
    /// Reads the claims of a JWT without checking its signature.
    ///
    /// # Errors
    ///
    /// [`TokenError`] when the token is not three dot-separated parts, its payload is not
    /// base64url (padded or not), or the payload lacks `iss`, `aud`, `sub`, `iat` or `exp`.
    ///
    /// ```
    /// use base64::Engine as _;
    /// use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    /// use hqf_godot_google_signin::IdTokenClaims;
    /// let payload = r#"{"iss":"accounts.google.com","aud":"a","sub":"1","iat":1,"exp":2}"#;
    /// let jwt = format!("h.{}.s", URL_SAFE_NO_PAD.encode(payload));
    /// assert_eq!(IdTokenClaims::from_jwt(&jwt).unwrap().sub, "1");
    /// ```
    pub fn from_jwt(token: &str) -> Result<Self, TokenError> {
        let parts: Vec<&str> = token.trim().split('.').collect();
        let [_, payload, _] = parts.as_slice() else {
            return Err(TokenError::Parts(parts.len()));
        };
        let bytes = URL_SAFE_NO_PAD
            .decode(payload.trim_end_matches('='))
            .map_err(|_| TokenError::Base64)?;
        serde_json::from_slice(&bytes).map_err(|error| TokenError::Json(error.to_string()))
    }

    /// Whether the token carries exactly this nonce.
    pub fn nonce_matches(&self, expected: &str) -> bool {
        self.nonce.as_deref() == Some(expected)
    }

    /// Whether the token is meant for this client ID.
    pub fn is_for(&self, client_id: &str) -> bool {
        self.aud.iter().any(|aud| aud == client_id)
    }

    /// Whether the token has expired at this Unix time, in seconds.
    pub const fn is_expired_at(&self, unix_seconds: i64) -> bool {
        unix_seconds >= self.exp
    }
}

fn one_or_many<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(String),
        Many(Vec<String>),
    }
    Ok(match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(one) => vec![one],
        OneOrMany::Many(many) => many,
    })
}

fn bool_or_text<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrText {
        Bool(bool),
        Text(String),
    }
    Ok(match Option::<BoolOrText>::deserialize(deserializer)? {
        None => None,
        Some(BoolOrText::Bool(value)) => Some(value),
        Some(BoolOrText::Text(text)) => match text.as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::engine::general_purpose::URL_SAFE;
    use proptest::prelude::*;

    fn jwt(payload: &str) -> String {
        format!(
            "eyJhbGciOiJSUzI1NiJ9.{}.sig",
            URL_SAFE_NO_PAD.encode(payload)
        )
    }

    const FULL: &str = r#"{
        "iss": "https://accounts.google.com",
        "aud": "web.apps.googleusercontent.com",
        "sub": "1234567890",
        "iat": 1700000000,
        "exp": 1700003600,
        "email": "player@example.com",
        "email_verified": true,
        "name": "Ada Lovelace",
        "given_name": "Ada",
        "family_name": "Lovelace",
        "picture": "https://example.com/p.png",
        "hd": "example.com",
        "nonce": "n0nce"
    }"#;

    #[test]
    fn reads_every_claim() {
        let claims = IdTokenClaims::from_jwt(&jwt(FULL)).unwrap();
        assert_eq!(claims.iss, "https://accounts.google.com");
        assert_eq!(claims.aud, ["web.apps.googleusercontent.com"]);
        assert_eq!(claims.sub, "1234567890");
        assert_eq!(claims.iat, 1_700_000_000);
        assert_eq!(claims.exp, 1_700_003_600);
        assert_eq!(claims.email.as_deref(), Some("player@example.com"));
        assert_eq!(claims.email_verified, Some(true));
        assert_eq!(claims.name.as_deref(), Some("Ada Lovelace"));
        assert_eq!(claims.given_name.as_deref(), Some("Ada"));
        assert_eq!(claims.family_name.as_deref(), Some("Lovelace"));
        assert_eq!(claims.picture.as_deref(), Some("https://example.com/p.png"));
        assert_eq!(claims.hd.as_deref(), Some("example.com"));
        assert_eq!(claims.nonce.as_deref(), Some("n0nce"));
    }

    #[test]
    fn optional_claims_may_be_missing() {
        let claims =
            IdTokenClaims::from_jwt(&jwt(r#"{"iss":"i","aud":"a","sub":"s","iat":0,"exp":1}"#))
                .unwrap();
        assert_eq!(claims.email, None);
        assert_eq!(claims.email_verified, None);
        assert_eq!(claims.nonce, None);
        assert!(!claims.nonce_matches(""));
    }

    #[test]
    fn audience_may_be_a_list() {
        let payload = r#"{"iss":"i","aud":["a","b"],"sub":"s","iat":0,"exp":1}"#;
        let claims = IdTokenClaims::from_jwt(&jwt(payload)).unwrap();
        assert_eq!(claims.aud, ["a", "b"]);
        assert!(claims.is_for("b"));
        assert!(!claims.is_for("c"));
    }

    #[test]
    fn email_verified_as_text() {
        for (text, value) in [("\"true\"", Some(true)), ("\"false\"", Some(false))] {
            let payload = format!(
                r#"{{"iss":"i","aud":"a","sub":"s","iat":0,"exp":1,"email_verified":{text}}}"#
            );
            let claims = IdTokenClaims::from_jwt(&jwt(&payload)).unwrap();
            assert_eq!(claims.email_verified, value, "{text}");
        }
        let payload = r#"{"iss":"i","aud":"a","sub":"s","iat":0,"exp":1,"email_verified":"yes"}"#;
        assert_eq!(
            IdTokenClaims::from_jwt(&jwt(payload))
                .unwrap()
                .email_verified,
            None
        );
        let payload = r#"{"iss":"i","aud":"a","sub":"s","iat":0,"exp":1,"email_verified":false}"#;
        assert_eq!(
            IdTokenClaims::from_jwt(&jwt(payload))
                .unwrap()
                .email_verified,
            Some(false)
        );
    }

    #[test]
    fn padded_payload_and_spaces_are_accepted() {
        let payload = r#"{"iss":"i","aud":"a","sub":"s","iat":0,"exp":1}"#;
        let token = format!(" h.{}.s\n", URL_SAFE.encode(payload));
        assert_eq!(IdTokenClaims::from_jwt(&token).unwrap().sub, "s");
    }

    #[test]
    fn wrong_part_counts() {
        for (token, parts) in [("", 1), ("a", 1), ("a.b", 2), ("a.b.c.d", 4)] {
            assert_eq!(
                IdTokenClaims::from_jwt(token),
                Err(TokenError::Parts(parts))
            );
        }
    }

    #[test]
    fn bad_base64_and_json() {
        assert_eq!(IdTokenClaims::from_jwt("h.!!.s"), Err(TokenError::Base64));
        let error = IdTokenClaims::from_jwt(&jwt("[]")).unwrap_err();
        assert!(matches!(error, TokenError::Json(_)), "{error:?}");
        let error = IdTokenClaims::from_jwt(&jwt(r#"{"iss":"i"}"#)).unwrap_err();
        assert!(
            error
                .to_string()
                .starts_with("payload is not a Google ID token: ")
        );
    }

    #[test]
    fn errors_become_invalid_token() {
        let error: SignInError = TokenError::Base64.into();
        assert_eq!(error.code, ErrorCode::InvalidToken);
        assert_eq!(error.message, "payload is not base64url");
        assert_eq!(
            TokenError::Parts(2).to_string(),
            "an ID token has 3 dot-separated parts, this one has 2"
        );
    }

    #[test]
    fn nonce_and_expiry() {
        let claims = IdTokenClaims::from_jwt(&jwt(FULL)).unwrap();
        assert!(claims.nonce_matches("n0nce"));
        assert!(!claims.nonce_matches("n0nc"));
        assert!(!claims.is_expired_at(1_700_003_599));
        assert!(claims.is_expired_at(1_700_003_600));
        assert!(claims.is_expired_at(1_700_003_601));
    }

    proptest! {
        #[test]
        fn never_panics(token in "\\PC*") {
            let _ = IdTokenClaims::from_jwt(&token);
        }

        #[test]
        fn claims_round_trip(
            sub in "[0-9]{1,21}",
            email in "[a-z]{1,10}@[a-z]{1,10}\\.com",
            nonce in "[A-Za-z0-9_-]{1,43}",
            exp in 0_i64..4_000_000_000,
        ) {
            let payload = serde_json::json!({
                "iss": "accounts.google.com", "aud": "a", "sub": sub, "iat": 0,
                "exp": exp, "email": email, "nonce": nonce,
            });
            let claims = IdTokenClaims::from_jwt(&jwt(&payload.to_string())).unwrap();
            prop_assert_eq!(&claims.sub, &sub);
            prop_assert_eq!(claims.email.as_deref(), Some(email.as_str()));
            prop_assert!(claims.nonce_matches(&nonce));
            prop_assert_eq!(claims.exp, exp);
        }
    }
}
