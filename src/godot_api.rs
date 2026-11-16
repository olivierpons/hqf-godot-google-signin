//! The Godot classes: the `HqfGoogleSignIn` node a game uses, the Java callback object
//! it hands to Android, and the [`JavaRuntime`] over Godot's `JavaClassWrapper`.
#![allow(
    clippy::doc_markdown,
    reason = "class docs are Godot BBCode shown in the editor, not rustdoc"
)]
#![allow(
    clippy::needless_pass_by_value,
    reason = "#[func] parameters are passed by value by Godot"
)]
#![allow(
    missing_docs,
    reason = "gdext generates undocumented getters and setters for #[export] fields"
)]

use crate::config::{self, Mode, SignInConfig};
use crate::error::{ErrorCode, SignInError};
use crate::flow::{Attempt, Flow, Next};
use crate::java::{self, CALLBACK_INTERFACE, Credential, JValue, JavaRuntime};
use godot::classes::{Engine, Node, Object, Os};
use godot::prelude::*;

const JAVA_CLASS_WRAPPER: &str = "JavaClassWrapper";
const ANDROID_RUNTIME: &str = "AndroidRuntime";

/// [`JavaRuntime`] over Godot's `JavaClassWrapper` singleton; handles are `JavaClass` and
/// `JavaObject` variants.
#[derive(Debug)]
struct GodotJava {
    wrapper: Gd<Object>,
}

impl GodotJava {
    fn new() -> Result<Self, SignInError> {
        if Os::singleton().get_name() != "Android" {
            return Err(SignInError::new(
                ErrorCode::UnsupportedPlatform,
                "Google Sign-In runs on Android only",
            ));
        }
        let wrapper = Engine::singleton()
            .get_singleton(JAVA_CLASS_WRAPPER)
            .ok_or_else(|| {
                SignInError::new(ErrorCode::UnsupportedPlatform, "no JavaClassWrapper")
            })?;
        Ok(Self { wrapper })
    }

    /// The exception the last Java call threw, as an error.
    fn thrown(&mut self, what: &str) -> Option<SignInError> {
        let exception = self.wrapper.try_call("get_exception", &[]).ok()?;
        if exception.is_nil() {
            return None;
        }
        let read = java::read_exception(self, &exception);
        Some(SignInError::new(
            ErrorCode::Bridge,
            format!("{what} threw {}: {}", read.code, read.message),
        ))
    }

    fn proxy(&mut self, callback: Gd<HqfCredentialCallback>) -> Result<Variant, SignInError> {
        let interfaces = PackedStringArray::from(&[GString::from(CALLBACK_INTERFACE)]);
        let proxy = self
            .wrapper
            .try_call(
                "create_proxy",
                &[callback.to_variant(), interfaces.to_variant()],
            )
            .map_err(|error| SignInError::new(ErrorCode::Bridge, error.to_string()))?;
        if proxy.is_nil() {
            return Err(SignInError::new(
                ErrorCode::Bridge,
                "JavaClassWrapper.create_proxy gave null",
            ));
        }
        Ok(proxy)
    }
}

fn to_variant(value: &JValue<Variant>) -> Variant {
    match value {
        JValue::Null => Variant::nil(),
        JValue::Bool(value) => value.to_variant(),
        JValue::Int(value) => value.to_variant(),
        JValue::Str(text) => text.to_variant(),
        JValue::Obj(object) => object.clone(),
    }
}

fn from_variant(value: Variant) -> JValue<Variant> {
    match value.get_type() {
        VariantType::NIL => JValue::Null,
        VariantType::BOOL => JValue::Bool(value.to::<bool>()),
        VariantType::INT => JValue::Int(value.to::<i64>()),
        VariantType::STRING | VariantType::STRING_NAME => JValue::Str(value.to_string()),
        _ => JValue::Obj(value),
    }
}

impl JavaRuntime for GodotJava {
    type Handle = Variant;

    fn class(&mut self, name: &str) -> Result<Variant, SignInError> {
        let class = self
            .wrapper
            .try_call("wrap", &[name.to_variant()])
            .map_err(|error| SignInError::new(ErrorCode::Bridge, error.to_string()))?;
        if class.is_nil() {
            return Err(SignInError::new(
                ErrorCode::Bridge,
                format!("Java class {name} not found"),
            ));
        }
        Ok(class)
    }

    fn call(
        &mut self,
        target: &Variant,
        method: &str,
        args: &[JValue<Variant>],
    ) -> Result<JValue<Variant>, SignInError> {
        let mut object = target.try_to::<Gd<Object>>().map_err(|_| {
            SignInError::new(ErrorCode::Bridge, format!("{method} on a non-object"))
        })?;
        let args: Vec<Variant> = args.iter().map(to_variant).collect();
        let result = object
            .try_call(method, &args)
            .map_err(|error| SignInError::new(ErrorCode::Bridge, error.to_string()))?;
        if method != "getClass"
            && method != "getName"
            && method != "getMessage"
            && let Some(error) = self.thrown(method)
        {
            return Err(error);
        }
        Ok(from_variant(result))
    }

    fn activity(&mut self) -> Result<Variant, SignInError> {
        let missing = || SignInError::new(ErrorCode::UnsupportedPlatform, "no Android activity");
        let mut runtime = Engine::singleton()
            .get_singleton(ANDROID_RUNTIME)
            .ok_or_else(missing)?;
        let activity = runtime
            .try_call("getActivity", &[])
            .map_err(|_| missing())?;
        if activity.is_nil() {
            return Err(missing());
        }
        Ok(activity)
    }
}

/// The object Android calls back with the answer to a request.
///
/// Android calls it on its main thread, not Godot's: it only forwards the answer to its
/// node with `call_deferred`, and the node never touches it once handed to Java.
#[derive(GodotClass)]
#[class(init, base = RefCounted)]
pub struct HqfCredentialCallback {
    target: Option<InstanceId>,
    on_result: StringName,
    on_error: StringName,
    ticket: i64,
}

#[godot_api]
impl HqfCredentialCallback {
    #[func(rename = onResult)]
    fn on_result(&self, value: Variant) {
        self.forward(&self.on_result, value);
    }

    #[func(rename = onError)]
    fn on_error(&self, exception: Variant) {
        self.forward(&self.on_error, exception);
    }
}

impl HqfCredentialCallback {
    fn forward(&self, method: &StringName, value: Variant) {
        let Some(target) = self.target else {
            return;
        };
        if let Ok(mut node) = Gd::<Object>::try_from_instance_id(target) {
            node.call_deferred(method, &[value, self.ticket.to_variant()]);
        }
    }
}

/// Signs the player in with their Google account, on Android.
///
/// Set [member web_client_id] to the Web application client ID of Google Cloud, call one
/// of the `sign_in*` methods, then wait for [signal signed_in] or
/// [signal sign_in_failed]. Every failure carries a stable code, see the README.
#[derive(GodotClass)]
#[class(init, base = Node)]
pub struct HqfGoogleSignIn {
    /// Web application OAuth client ID (`<id>.apps.googleusercontent.com`).
    #[export]
    web_client_id: GString,
    /// Only offer accounts of this Google Workspace domain; empty offers every account.
    #[export]
    hosted_domain: GString,
    /// Ask Google for a verified phone number.
    #[export]
    request_verified_phone_number: bool,
    flow: Flow,
    config: Option<SignInConfig>,
    nonce: String,
    cancel_signal: Option<Variant>,
    ticket: i64,
    signing_out: bool,
    owner: Base<Node>,
}

#[godot_api]
impl HqfGoogleSignIn {
    /// The player signed in. [param account] holds [code]id_token[/code],
    /// [code]email[/code], [code]display_name[/code], [code]given_name[/code],
    /// [code]family_name[/code], [code]picture[/code], [code]phone_number[/code] and the
    /// [code]nonce[/code] the request carried.
    #[signal]
    fn signed_in(account: VarDictionary);

    /// The sign-in failed; [param code] is stable (e.g. [code]cancelled[/code],
    /// [code]no_credential[/code]), [param message] is detail for logs.
    #[signal]
    fn sign_in_failed(code: GString, message: GString);

    /// The Credential Manager forgot the signed-in account.
    #[signal]
    fn signed_out();

    /// Signing out failed.
    #[signal]
    fn sign_out_failed(code: GString, message: GString);

    /// Accounts that already signed in first, every account when none did. An empty
    /// [param nonce] is generated.
    #[func]
    fn sign_in(&mut self, nonce: GString) {
        self.begin(Mode::Automatic, &nonce.to_string());
    }

    /// Every Google account of the device.
    #[func]
    fn sign_in_with_account_chooser(&mut self, nonce: GString) {
        self.begin(Mode::AccountChooser, &nonce.to_string());
    }

    /// The flow meant to follow a press on a "Sign in with Google" button.
    #[func]
    fn sign_in_with_button(&mut self, nonce: GString) {
        self.begin(Mode::Button, &nonce.to_string());
    }

    /// Accounts that already signed in only; fails with [code]no_credential[/code] when
    /// there are none.
    #[func]
    fn sign_in_silently(&mut self, nonce: GString) {
        self.begin(Mode::Silent, &nonce.to_string());
    }

    /// Whether a sign-in waits for its answer.
    #[func]
    #[allow(clippy::missing_const_for_fn, reason = "a #[func] cannot be const")]
    fn is_signing_in(&self) -> bool {
        self.flow.is_running()
    }

    /// Whether sign-in can work here: Android with Godot's Java bridge.
    #[func]
    #[allow(clippy::unused_self, reason = "a #[func] is called on the node")]
    fn is_supported(&self) -> bool {
        GodotJava::new().is_ok()
    }

    /// Stops the running sign-in; emits [signal sign_in_failed] with
    /// [code]cancelled[/code].
    #[func]
    fn cancel(&mut self) {
        if !self.flow.is_running() {
            return;
        }
        if let (Some(signal), Ok(mut runtime)) = (self.cancel_signal.take(), GodotJava::new()) {
            let _ = runtime.call(&signal, "cancel", &[]);
        }
        self.ticket += 1;
        self.flow.cancel();
        self.fail(&SignInError::new(
            ErrorCode::Cancelled,
            "cancelled by the game",
        ));
    }

    /// Makes the Credential Manager forget the signed-in account, so that [method sign_in]
    /// shows the account sheet again.
    #[func]
    fn sign_out(&mut self) {
        if self.signing_out {
            return;
        }
        let sent = GodotJava::new().and_then(|mut runtime| {
            let callback = self.callback("_on_sign_out_result", "_on_sign_out_error");
            let proxy = runtime.proxy(callback)?;
            java::send_sign_out(&mut runtime, proxy)
        });
        match sent {
            Ok(()) => self.signing_out = true,
            Err(error) => self.sign_out_fail(&error),
        }
    }

    #[func]
    fn _on_credential_result(&mut self, response: Variant, ticket: i64) {
        if ticket != self.ticket || !self.flow.is_running() {
            return;
        }
        self.cancel_signal = None;
        let read =
            GodotJava::new().and_then(|mut runtime| java::read_response(&mut runtime, &response));
        let _ = self.flow.succeed();
        match read {
            Ok(credential) => self.succeed(&credential),
            Err(error) => self.fail(&error),
        }
    }

    #[func]
    fn _on_credential_error(&mut self, exception: Variant, ticket: i64) {
        if ticket != self.ticket || !self.flow.is_running() {
            return;
        }
        self.cancel_signal = None;
        let error = match GodotJava::new() {
            Ok(mut runtime) => java::read_exception(&mut runtime, &exception),
            Err(error) => error,
        };
        match self.flow.fail(error) {
            Next::Request(attempt) => self.send(attempt),
            Next::Fail(error) => self.fail(&error),
        }
    }

    #[func]
    fn _on_sign_out_result(&mut self, _value: Variant, _ticket: i64) {
        self.signing_out = false;
        self.signals().signed_out().emit();
    }

    #[func]
    fn _on_sign_out_error(&mut self, exception: Variant, _ticket: i64) {
        self.signing_out = false;
        let error = match GodotJava::new() {
            Ok(mut runtime) => java::read_exception(&mut runtime, &exception),
            Err(error) => error,
        };
        self.sign_out_fail(&error);
    }
}

impl HqfGoogleSignIn {
    fn begin(&mut self, mode: Mode, nonce: &str) {
        let prepared = GodotJava::new().and_then(|_| {
            let config = SignInConfig::new(&self.web_client_id.to_string())?
                .with_hosted_domain(&self.hosted_domain.to_string())
                .with_verified_phone_number(self.request_verified_phone_number);
            Ok((config, config::nonce(nonce)?))
        });
        let (config, nonce) = match prepared {
            Ok(prepared) => prepared,
            Err(error) => return self.fail(&error),
        };
        match self.flow.start(mode) {
            Ok(attempt) => {
                self.config = Some(config);
                self.nonce = nonce;
                self.send(attempt);
            }
            Err(error) => self.fail(&error),
        }
    }

    fn send(&mut self, attempt: Attempt) {
        self.ticket += 1;
        let Some(config) = self.config.clone() else {
            self.flow.cancel();
            return self.fail(&SignInError::new(ErrorCode::InvalidConfig, "no config"));
        };
        let callback = self.callback("_on_credential_result", "_on_credential_error");
        let sent = GodotJava::new().and_then(|mut runtime| {
            let option = java::build_option(&mut runtime, &config, attempt, &self.nonce)?;
            let request = java::build_request(&mut runtime, option)?;
            let proxy = runtime.proxy(callback)?;
            java::send_request(&mut runtime, request, proxy)
        });
        match sent {
            Ok(signal) => self.cancel_signal = Some(signal),
            Err(error) => {
                self.flow.cancel();
                self.fail(&error);
            }
        }
    }

    fn callback(&self, on_result: &str, on_error: &str) -> Gd<HqfCredentialCallback> {
        let mut callback = HqfCredentialCallback::new_gd();
        {
            let mut fields = callback.bind_mut();
            fields.target = Some(self.base().instance_id());
            fields.on_result = StringName::from(on_result);
            fields.on_error = StringName::from(on_error);
            fields.ticket = self.ticket;
        }
        callback
    }

    fn succeed(&mut self, credential: &Credential) {
        let mut account = VarDictionary::new();
        for (key, value) in [
            ("id_token", &credential.id_token),
            ("email", &credential.email),
            ("display_name", &credential.display_name),
            ("given_name", &credential.given_name),
            ("family_name", &credential.family_name),
            ("picture", &credential.picture),
            ("phone_number", &credential.phone_number),
            ("nonce", &self.nonce),
        ] {
            account.set(key, value.as_str());
        }
        self.signals().signed_in().emit(&account);
    }

    fn fail(&mut self, error: &SignInError) {
        let code = GString::from(error.code.as_str());
        let message = GString::from(error.message.as_str());
        self.signals().sign_in_failed().emit(&code, &message);
    }

    fn sign_out_fail(&mut self, error: &SignInError) {
        let code = GString::from(error.code.as_str());
        let message = GString::from(error.message.as_str());
        self.signals().sign_out_failed().emit(&code, &message);
    }
}
