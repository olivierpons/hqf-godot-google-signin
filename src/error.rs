//! Why a sign-in or a sign-out did not succeed, as stable codes a game can match on.

use std::fmt;

/// Why a sign-in or a sign-out did not succeed.
///
/// [`ErrorCode::as_str`] gives the stable snake-case code sent to GDScript.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorCode {
    /// The player closed the account sheet or pressed back.
    Cancelled,
    /// No Google account matches the request on this device.
    NoCredential,
    /// The request was interrupted before it finished; it may be retried.
    Interrupted,
    /// The device has no credential provider set up for Google accounts.
    ProviderConfiguration,
    /// The device does not support the Credential Manager API.
    Unsupported,
    /// The credential provider failed without saying why.
    Unknown,
    /// The credential provider raised an error of its own kind.
    Custom,
    /// The provider answered with something that is not a Google ID token.
    UnexpectedCredential,
    /// The Google ID token could not be read.
    InvalidToken,
    /// The configuration given to the plugin is not usable.
    InvalidConfig,
    /// A sign-in or a sign-out is already running.
    Busy,
    /// Not running on Android, or the Godot Android runtime is missing.
    UnsupportedPlatform,
    /// A call into the Android API failed.
    Bridge,
}

impl ErrorCode {
    /// Every code, in declaration order.
    pub const ALL: [Self; 13] = [
        Self::Cancelled,
        Self::NoCredential,
        Self::Interrupted,
        Self::ProviderConfiguration,
        Self::Unsupported,
        Self::Unknown,
        Self::Custom,
        Self::UnexpectedCredential,
        Self::InvalidToken,
        Self::InvalidConfig,
        Self::Busy,
        Self::UnsupportedPlatform,
        Self::Bridge,
    ];

    /// The stable snake-case code of this error.
    ///
    /// ```
    /// use hqf_godot_google_signin::ErrorCode;
    /// assert_eq!(ErrorCode::NoCredential.as_str(), "no_credential");
    /// ```
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cancelled => "cancelled",
            Self::NoCredential => "no_credential",
            Self::Interrupted => "interrupted",
            Self::ProviderConfiguration => "provider_configuration",
            Self::Unsupported => "unsupported",
            Self::Unknown => "unknown",
            Self::Custom => "custom",
            Self::UnexpectedCredential => "unexpected_credential",
            Self::InvalidToken => "invalid_token",
            Self::InvalidConfig => "invalid_config",
            Self::Busy => "busy",
            Self::UnsupportedPlatform => "unsupported_platform",
            Self::Bridge => "bridge",
        }
    }

    /// The code of an `androidx.credentials` exception, from its Java class name.
    ///
    /// Both `GetCredential*` and `ClearCredential*` exceptions are known; any other class
    /// name gives [`ErrorCode::Unknown`]. The package part of the name is ignored.
    ///
    /// ```
    /// use hqf_godot_google_signin::ErrorCode;
    /// let name = "androidx.credentials.exceptions.GetCredentialCancellationException";
    /// assert_eq!(ErrorCode::from_exception_class(name), ErrorCode::Cancelled);
    /// ```
    pub fn from_exception_class(class_name: &str) -> Self {
        let simple = class_name.rsplit(['.', '$']).next().unwrap_or(class_name);
        match simple {
            "GetCredentialCancellationException" => Self::Cancelled,
            "NoCredentialException" => Self::NoCredential,
            "GetCredentialInterruptedException" | "ClearCredentialInterruptedException" => {
                Self::Interrupted
            }
            "GetCredentialProviderConfigurationException"
            | "ClearCredentialProviderConfigurationException" => Self::ProviderConfiguration,
            "GetCredentialUnsupportedException" | "ClearCredentialUnsupportedException" => {
                Self::Unsupported
            }
            "GetCredentialCustomException" | "ClearCredentialCustomException" => Self::Custom,
            _ => Self::Unknown,
        }
    }

    /// Whether retrying the same request may succeed without the player doing anything.
    pub const fn is_retryable(self) -> bool {
        matches!(self, Self::Interrupted)
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// An error code with the detail message that came with it.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("{code}: {message}")]
pub struct SignInError {
    /// What went wrong.
    pub code: ErrorCode,
    /// Detail for logs, in English, possibly empty.
    pub message: String,
}

impl SignInError {
    /// An error with its detail message.
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn codes_are_unique_snake_case() {
        let codes: HashSet<&str> = ErrorCode::ALL.iter().map(|c| c.as_str()).collect();
        assert_eq!(codes.len(), ErrorCode::ALL.len());
        for code in codes {
            assert!(
                code.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{code}"
            );
        }
    }

    #[test]
    fn display_is_the_code() {
        for code in ErrorCode::ALL {
            assert_eq!(code.to_string(), code.as_str());
        }
    }

    #[test]
    fn every_known_exception_class() {
        let cases = [
            ("GetCredentialCancellationException", ErrorCode::Cancelled),
            ("NoCredentialException", ErrorCode::NoCredential),
            ("GetCredentialInterruptedException", ErrorCode::Interrupted),
            (
                "ClearCredentialInterruptedException",
                ErrorCode::Interrupted,
            ),
            (
                "GetCredentialProviderConfigurationException",
                ErrorCode::ProviderConfiguration,
            ),
            (
                "ClearCredentialProviderConfigurationException",
                ErrorCode::ProviderConfiguration,
            ),
            ("GetCredentialUnsupportedException", ErrorCode::Unsupported),
            (
                "ClearCredentialUnsupportedException",
                ErrorCode::Unsupported,
            ),
            ("GetCredentialCustomException", ErrorCode::Custom),
            ("ClearCredentialCustomException", ErrorCode::Custom),
            ("GetCredentialUnknownException", ErrorCode::Unknown),
            ("ClearCredentialUnknownException", ErrorCode::Unknown),
            ("java.lang.IllegalStateException", ErrorCode::Unknown),
            ("", ErrorCode::Unknown),
        ];
        for (name, code) in cases {
            let full = format!("androidx.credentials.exceptions.{name}");
            assert_eq!(ErrorCode::from_exception_class(name), code, "{name}");
            assert_eq!(ErrorCode::from_exception_class(&full), code, "{full}");
        }
    }

    #[test]
    fn nested_class_names_use_the_last_part() {
        let name = "a.b.Outer$NoCredentialException";
        assert_eq!(
            ErrorCode::from_exception_class(name),
            ErrorCode::NoCredential
        );
    }

    #[test]
    fn only_interrupted_is_retryable() {
        for code in ErrorCode::ALL {
            assert_eq!(
                code.is_retryable(),
                code == ErrorCode::Interrupted,
                "{code}"
            );
        }
    }

    #[test]
    fn sign_in_error_display() {
        let error = SignInError::new(ErrorCode::Busy, "already running");
        assert_eq!(error.to_string(), "busy: already running");
    }
}
