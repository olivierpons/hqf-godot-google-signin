//! The Credential Manager calls of a sign-in, written against [`JavaRuntime`] so that
//! they run on Godot's `JavaClassWrapper` on Android and on a recording fake in tests.

use crate::config::SignInConfig;
use crate::error::{ErrorCode, SignInError};
use crate::flow::Attempt;
use std::fmt;

/// Java class of the `GetGoogleIdOption` builder.
pub const GOOGLE_ID_OPTION_BUILDER: &str =
    "com.google.android.libraries.identity.googleid.GetGoogleIdOption$Builder";
/// Java class of the `GetSignInWithGoogleOption` builder.
pub const SIGN_IN_WITH_GOOGLE_OPTION_BUILDER: &str =
    "com.google.android.libraries.identity.googleid.GetSignInWithGoogleOption$Builder";
/// Java class of a Google ID token credential.
pub const GOOGLE_ID_TOKEN_CREDENTIAL: &str =
    "com.google.android.libraries.identity.googleid.GoogleIdTokenCredential";
/// Credential type of a Google ID token.
pub const TYPE_GOOGLE_ID_TOKEN: &str =
    "com.google.android.libraries.identity.googleid.TYPE_GOOGLE_ID_TOKEN_CREDENTIAL";
/// Credential type of a Google ID token given by the Sign in with Google flow.
pub const TYPE_GOOGLE_ID_TOKEN_SIWG: &str =
    "com.google.android.libraries.identity.googleid.TYPE_GOOGLE_ID_TOKEN_SIWG_CREDENTIAL";
/// Java class of the request builder.
pub const GET_CREDENTIAL_REQUEST_BUILDER: &str =
    "androidx.credentials.GetCredentialRequest$Builder";
/// Java class of the Credential Manager.
pub const CREDENTIAL_MANAGER: &str = "androidx.credentials.CredentialManager";
/// Java class of the sign-out request.
pub const CLEAR_CREDENTIAL_STATE_REQUEST: &str = "androidx.credentials.ClearCredentialStateRequest";
/// Java class of a cancellation signal.
pub const CANCELLATION_SIGNAL: &str = "android.os.CancellationSignal";
/// Java interface the answer callback implements.
pub const CALLBACK_INTERFACE: &str = "androidx.credentials.CredentialManagerCallback";

/// A value passed to or returned by a Java call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JValue<H> {
    /// Java `null`, or a `void` return.
    Null,
    /// A Java `boolean`.
    Bool(bool),
    /// A Java `int` or `long`.
    Int(i64),
    /// A Java `String`.
    Str(String),
    /// Any other Java object.
    Obj(H),
}

impl<H> JValue<H> {
    fn into_string(self) -> Option<String> {
        match self {
            Self::Str(text) => Some(text),
            _ => None,
        }
    }
}

/// Calls into Java.
pub trait JavaRuntime {
    /// A Java object or class held by the runtime.
    type Handle: Clone + fmt::Debug;

    /// The class of this binary name (`$` for nested classes).
    ///
    /// # Errors
    ///
    /// [`ErrorCode::Bridge`] when the class is not in the app.
    fn class(&mut self, name: &str) -> Result<Self::Handle, SignInError>;

    /// Calls `method` on `target`, an object or a class (static methods; a method named
    /// like the class's last name part is its constructor).
    ///
    /// # Errors
    ///
    /// [`ErrorCode::Bridge`] when the call fails or throws.
    fn call(
        &mut self,
        target: &Self::Handle,
        method: &str,
        args: &[JValue<Self::Handle>],
    ) -> Result<JValue<Self::Handle>, SignInError>;

    /// The running Android activity.
    ///
    /// # Errors
    ///
    /// [`ErrorCode::UnsupportedPlatform`] when there is none.
    fn activity(&mut self) -> Result<Self::Handle, SignInError>;
}

/// What a successful sign-in gives the game; empty strings for what Google left out.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Credential {
    /// The Google ID token, a JWT for the game's server to verify.
    pub id_token: String,
    /// The account's email address.
    pub email: String,
    /// Full name.
    pub display_name: String,
    /// First name.
    pub given_name: String,
    /// Last name.
    pub family_name: String,
    /// URL of the profile picture.
    pub picture: String,
    /// Verified phone number, when asked for and given.
    pub phone_number: String,
}

fn bridge(message: impl Into<String>) -> SignInError {
    SignInError::new(ErrorCode::Bridge, message)
}

fn object<H>(value: JValue<H>, what: &str) -> Result<H, SignInError> {
    match value {
        JValue::Obj(handle) => Ok(handle),
        _ => Err(bridge(format!("{what} gave no object"))),
    }
}

/// Creates an object of class `name` with `args`.
///
/// # Errors
///
/// [`ErrorCode::Bridge`] when the class is missing or the constructor fails.
pub fn construct<R: JavaRuntime>(
    runtime: &mut R,
    name: &str,
    args: &[JValue<R::Handle>],
) -> Result<R::Handle, SignInError> {
    let class = runtime.class(name)?;
    let constructor = name.rsplit('.').next().unwrap_or(name);
    let value = runtime.call(&class, constructor, args)?;
    object(value, name)
}

/// Builds the credential option of `attempt`.
///
/// # Errors
///
/// [`ErrorCode::Bridge`] when a Java call fails.
pub fn build_option<R: JavaRuntime>(
    runtime: &mut R,
    config: &SignInConfig,
    attempt: Attempt,
    nonce: &str,
) -> Result<R::Handle, SignInError> {
    let client_id = JValue::Str(config.web_client_id().to_owned());
    let builder = match attempt {
        Attempt::GoogleId {
            filter_by_authorized_accounts,
            auto_select,
        } => {
            let builder = construct(runtime, GOOGLE_ID_OPTION_BUILDER, &[])?;
            runtime.call(&builder, "setServerClientId", &[client_id])?;
            runtime.call(
                &builder,
                "setFilterByAuthorizedAccounts",
                &[JValue::Bool(filter_by_authorized_accounts)],
            )?;
            runtime.call(
                &builder,
                "setAutoSelectEnabled",
                &[JValue::Bool(auto_select)],
            )?;
            if config.request_verified_phone_number() {
                runtime.call(
                    &builder,
                    "setRequestVerifiedPhoneNumber",
                    &[JValue::Bool(true)],
                )?;
            }
            builder
        }
        Attempt::SignInWithGoogle => {
            construct(runtime, SIGN_IN_WITH_GOOGLE_OPTION_BUILDER, &[client_id])?
        }
    };
    runtime.call(&builder, "setNonce", &[JValue::Str(nonce.to_owned())])?;
    if let Some(domain) = config.hosted_domain() {
        runtime.call(
            &builder,
            "setHostedDomainFilter",
            &[JValue::Str(domain.to_owned())],
        )?;
    }
    let option = runtime.call(&builder, "build", &[])?;
    object(option, "option build")
}

/// Builds the `GetCredentialRequest` holding `option`.
///
/// # Errors
///
/// [`ErrorCode::Bridge`] when a Java call fails.
pub fn build_request<R: JavaRuntime>(
    runtime: &mut R,
    option: R::Handle,
) -> Result<R::Handle, SignInError> {
    let builder = construct(runtime, GET_CREDENTIAL_REQUEST_BUILDER, &[])?;
    runtime.call(&builder, "addCredentialOption", &[JValue::Obj(option)])?;
    let request = runtime.call(&builder, "build", &[])?;
    object(request, "request build")
}

/// The running activity, its Credential Manager and its main executor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manager<H> {
    /// The running activity.
    pub activity: H,
    /// Its `CredentialManager`.
    pub manager: H,
    /// Its main thread executor, where answers arrive.
    pub executor: H,
}

/// The Credential Manager of the running activity, and the activity's main executor.
///
/// # Errors
///
/// [`ErrorCode::UnsupportedPlatform`] without an activity, [`ErrorCode::Bridge`] when a
/// Java call fails.
pub fn manager<R: JavaRuntime>(runtime: &mut R) -> Result<Manager<R::Handle>, SignInError> {
    let activity = runtime.activity()?;
    let class = runtime.class(CREDENTIAL_MANAGER)?;
    let manager = runtime.call(&class, "create", &[JValue::Obj(activity.clone())])?;
    let manager = object(manager, "CredentialManager.create")?;
    let executor = runtime.call(&activity, "getMainExecutor", &[])?;
    let executor = object(executor, "getMainExecutor")?;
    Ok(Manager {
        activity,
        manager,
        executor,
    })
}

/// Sends `request`; the answer goes to `callback`, a `CredentialManagerCallback`.
/// Returns the signal that cancels it.
///
/// # Errors
///
/// [`ErrorCode::Bridge`] or [`ErrorCode::UnsupportedPlatform`], see [`manager`].
pub fn send_request<R: JavaRuntime>(
    runtime: &mut R,
    request: R::Handle,
    callback: R::Handle,
) -> Result<R::Handle, SignInError> {
    let Manager {
        activity,
        manager,
        executor,
    } = manager(runtime)?;
    let signal = construct(runtime, CANCELLATION_SIGNAL, &[])?;
    runtime.call(
        &manager,
        "getCredentialAsync",
        &[
            JValue::Obj(activity),
            JValue::Obj(request),
            JValue::Obj(signal.clone()),
            JValue::Obj(executor),
            JValue::Obj(callback),
        ],
    )?;
    Ok(signal)
}

/// Clears the signed-in state of the Credential Manager; the answer goes to `callback`.
///
/// # Errors
///
/// [`ErrorCode::Bridge`] or [`ErrorCode::UnsupportedPlatform`], see [`manager`].
pub fn send_sign_out<R: JavaRuntime>(
    runtime: &mut R,
    callback: R::Handle,
) -> Result<(), SignInError> {
    let Manager {
        manager, executor, ..
    } = manager(runtime)?;
    let request = construct(runtime, CLEAR_CREDENTIAL_STATE_REQUEST, &[])?;
    runtime.call(
        &manager,
        "clearCredentialStateAsync",
        &[
            JValue::Obj(request),
            JValue::Null,
            JValue::Obj(executor),
            JValue::Obj(callback),
        ],
    )?;
    Ok(())
}

fn text<R: JavaRuntime>(
    runtime: &mut R,
    target: &R::Handle,
    getter: &str,
) -> Result<String, SignInError> {
    Ok(runtime
        .call(target, getter, &[])?
        .into_string()
        .unwrap_or_default())
}

/// Reads the Google credential of a `GetCredentialResponse`.
///
/// # Errors
///
/// [`ErrorCode::UnexpectedCredential`] when it is not a Google ID token,
/// [`ErrorCode::InvalidToken`] when its token is empty, [`ErrorCode::Bridge`] when a Java
/// call fails.
pub fn read_response<R: JavaRuntime>(
    runtime: &mut R,
    response: &R::Handle,
) -> Result<Credential, SignInError> {
    let credential = runtime.call(response, "getCredential", &[])?;
    let credential = object(credential, "getCredential")?;
    let kind = text(runtime, &credential, "getType")?;
    if kind != TYPE_GOOGLE_ID_TOKEN && kind != TYPE_GOOGLE_ID_TOKEN_SIWG {
        return Err(SignInError::new(
            ErrorCode::UnexpectedCredential,
            format!("credential type {kind}"),
        ));
    }
    let data = runtime.call(&credential, "getData", &[])?;
    let class = runtime.class(GOOGLE_ID_TOKEN_CREDENTIAL)?;
    let google = runtime.call(&class, "createFrom", &[data])?;
    let google = object(google, "GoogleIdTokenCredential.createFrom")?;
    let picture = match runtime.call(&google, "getProfilePictureUri", &[])? {
        JValue::Obj(uri) => text(runtime, &uri, "toString")?,
        _ => String::new(),
    };
    let result = Credential {
        id_token: text(runtime, &google, "getIdToken")?,
        email: text(runtime, &google, "getId")?,
        display_name: text(runtime, &google, "getDisplayName")?,
        given_name: text(runtime, &google, "getGivenName")?,
        family_name: text(runtime, &google, "getFamilyName")?,
        picture,
        phone_number: text(runtime, &google, "getPhoneNumber")?,
    };
    if result.id_token.is_empty() {
        return Err(SignInError::new(
            ErrorCode::InvalidToken,
            "empty Google ID token",
        ));
    }
    Ok(result)
}

/// The error of a Credential Manager exception object.
///
/// Its code comes from its Java class name, see [`ErrorCode::from_exception_class`]; its
/// message from `getMessage()`. A failure to read either gives [`ErrorCode::Unknown`].
pub fn read_exception<R: JavaRuntime>(runtime: &mut R, exception: &R::Handle) -> SignInError {
    let name = runtime
        .call(exception, "getClass", &[])
        .and_then(|class| object(class, "getClass"))
        .and_then(|class| text(runtime, &class, "getName"))
        .unwrap_or_default();
    let message = text(runtime, exception, "getMessage").unwrap_or_default();
    SignInError::new(ErrorCode::from_exception_class(&name), message)
}

#[cfg(test)]
mod tests;
