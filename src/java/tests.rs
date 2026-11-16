use super::*;
use crate::config::SignInConfig;
use std::collections::HashMap;

/// Records every call; a call returns the answer scripted for its method name, else a
/// new object named after the call.
#[derive(Default)]
struct Fake {
    names: Vec<String>,
    log: Vec<String>,
    answers: HashMap<&'static str, JValue<usize>>,
    fail_on: Option<&'static str>,
    no_activity: bool,
}

impl Fake {
    fn new_object(&mut self, name: String) -> usize {
        self.names.push(name);
        self.names.len() - 1
    }

    fn show(&self, value: &JValue<usize>) -> String {
        match value {
            JValue::Null => "null".to_owned(),
            JValue::Bool(value) => value.to_string(),
            JValue::Int(value) => value.to_string(),
            JValue::Str(text) => format!("{text:?}"),
            JValue::Obj(handle) => self.names[*handle].clone(),
        }
    }
}

impl JavaRuntime for Fake {
    type Handle = usize;

    fn class(&mut self, name: &str) -> Result<usize, SignInError> {
        if self.fail_on == Some("class") {
            return Err(SignInError::new(ErrorCode::Bridge, "no class"));
        }
        let short = name.rsplit('.').next().unwrap_or(name);
        Ok(self.new_object(format!("class {short}")))
    }

    fn call(
        &mut self,
        target: &usize,
        method: &str,
        args: &[JValue<usize>],
    ) -> Result<JValue<usize>, SignInError> {
        let shown: Vec<String> = args.iter().map(|arg| self.show(arg)).collect();
        let line = format!("{}.{method}({})", self.names[*target], shown.join(", "));
        self.log.push(line);
        if self.fail_on == Some(method) {
            return Err(SignInError::new(
                ErrorCode::Bridge,
                format!("{method} threw"),
            ));
        }
        if let Some(answer) = self.answers.get(method) {
            return Ok(answer.clone());
        }
        let object = self.new_object(format!("<{method}>"));
        Ok(JValue::Obj(object))
    }

    fn activity(&mut self) -> Result<usize, SignInError> {
        if self.no_activity {
            return Err(SignInError::new(
                ErrorCode::UnsupportedPlatform,
                "no activity",
            ));
        }
        Ok(self.new_object("activity".to_owned()))
    }
}

const ID: &str = "1-a.apps.googleusercontent.com";

fn config() -> SignInConfig {
    SignInConfig::new(ID).unwrap()
}

#[test]
fn google_id_option_calls() {
    let mut fake = Fake::default();
    let attempt = Attempt::GoogleId {
        filter_by_authorized_accounts: true,
        auto_select: false,
    };
    let option = build_option(&mut fake, &config(), attempt, "N").unwrap();
    assert_eq!(fake.names[option], "<build>");
    assert_eq!(
        fake.log,
        [
            "class GetGoogleIdOption$Builder.GetGoogleIdOption$Builder()",
            &format!("<GetGoogleIdOption$Builder>.setServerClientId({ID:?})"),
            "<GetGoogleIdOption$Builder>.setFilterByAuthorizedAccounts(true)",
            "<GetGoogleIdOption$Builder>.setAutoSelectEnabled(false)",
            "<GetGoogleIdOption$Builder>.setNonce(\"N\")",
            "<GetGoogleIdOption$Builder>.build()",
        ]
    );
}

#[test]
fn google_id_option_with_domain_and_phone() {
    let mut fake = Fake::default();
    let config = config()
        .with_hosted_domain("example.com")
        .with_verified_phone_number(true);
    let attempt = Attempt::GoogleId {
        filter_by_authorized_accounts: false,
        auto_select: true,
    };
    build_option(&mut fake, &config, attempt, "N").unwrap();
    assert_eq!(
        fake.log[2..],
        [
            "<GetGoogleIdOption$Builder>.setFilterByAuthorizedAccounts(false)",
            "<GetGoogleIdOption$Builder>.setAutoSelectEnabled(true)",
            "<GetGoogleIdOption$Builder>.setRequestVerifiedPhoneNumber(true)",
            "<GetGoogleIdOption$Builder>.setNonce(\"N\")",
            "<GetGoogleIdOption$Builder>.setHostedDomainFilter(\"example.com\")",
            "<GetGoogleIdOption$Builder>.build()",
        ]
    );
}

#[test]
fn sign_in_with_google_option_calls() {
    let mut fake = Fake::default();
    let config = config()
        .with_hosted_domain("example.com")
        .with_verified_phone_number(true);
    build_option(&mut fake, &config, Attempt::SignInWithGoogle, "N").unwrap();
    assert_eq!(
        fake.log,
        [
            &format!(
                "class GetSignInWithGoogleOption$Builder.GetSignInWithGoogleOption$Builder({ID:?})"
            ),
            "<GetSignInWithGoogleOption$Builder>.setNonce(\"N\")",
            "<GetSignInWithGoogleOption$Builder>.setHostedDomainFilter(\"example.com\")",
            "<GetSignInWithGoogleOption$Builder>.build()",
        ]
    );
}

#[test]
fn option_build_must_give_an_object() {
    let mut fake = Fake::default();
    fake.answers.insert("build", JValue::Null);
    let error = build_option(&mut fake, &config(), Attempt::SignInWithGoogle, "N").unwrap_err();
    assert_eq!(
        error,
        SignInError::new(ErrorCode::Bridge, "option build gave no object")
    );
}

#[test]
fn request_calls() {
    let mut fake = Fake::default();
    let option = fake.new_object("option".to_owned());
    let request = build_request(&mut fake, option).unwrap();
    assert_eq!(fake.names[request], "<build>");
    assert_eq!(
        fake.log,
        [
            "class GetCredentialRequest$Builder.GetCredentialRequest$Builder()",
            "<GetCredentialRequest$Builder>.addCredentialOption(option)",
            "<GetCredentialRequest$Builder>.build()",
        ]
    );
}

#[test]
fn request_build_must_give_an_object() {
    let mut fake = Fake::default();
    fake.answers.insert("build", JValue::Bool(true));
    let option = fake.new_object("option".to_owned());
    let error = build_request(&mut fake, option).unwrap_err();
    assert_eq!(error.message, "request build gave no object");
}

#[test]
fn send_request_calls() {
    let mut fake = Fake::default();
    let request = fake.new_object("request".to_owned());
    let callback = fake.new_object("callback".to_owned());
    let signal = send_request(&mut fake, request, callback).unwrap();
    assert_eq!(fake.names[signal], "<CancellationSignal>");
    assert_eq!(
        fake.log,
        [
            "class CredentialManager.create(activity)",
            "activity.getMainExecutor()",
            "class CancellationSignal.CancellationSignal()",
            "<create>.getCredentialAsync(activity, request, <CancellationSignal>, \
             <getMainExecutor>, callback)",
        ]
    );
}

#[test]
fn manager_needs_objects() {
    for (method, message) in [
        ("create", "CredentialManager.create gave no object"),
        ("getMainExecutor", "getMainExecutor gave no object"),
    ] {
        let mut fake = Fake::default();
        fake.answers.insert(method, JValue::Null);
        assert_eq!(manager(&mut fake).unwrap_err().message, message);
    }
}

#[test]
fn constructor_must_give_an_object() {
    let mut fake = Fake::default();
    fake.answers.insert("CancellationSignal", JValue::Null);
    let error = construct(&mut fake, CANCELLATION_SIGNAL, &[]).unwrap_err();
    assert_eq!(
        error.message,
        "android.os.CancellationSignal gave no object"
    );
}

#[test]
fn no_activity_is_unsupported_platform() {
    let mut fake = Fake {
        no_activity: true,
        ..Fake::default()
    };
    let request = fake.new_object("request".to_owned());
    let callback = fake.new_object("callback".to_owned());
    let error = send_request(&mut fake, request, callback).unwrap_err();
    assert_eq!(error.code, ErrorCode::UnsupportedPlatform);
    assert_eq!(fake.log, Vec::<String>::new());
}

#[test]
fn sign_out_calls() {
    let mut fake = Fake::default();
    let callback = fake.new_object("callback".to_owned());
    send_sign_out(&mut fake, callback).unwrap();
    assert_eq!(
        fake.log,
        [
            "class CredentialManager.create(activity)",
            "activity.getMainExecutor()",
            "class ClearCredentialStateRequest.ClearCredentialStateRequest()",
            "<create>.clearCredentialStateAsync(<ClearCredentialStateRequest>, null, \
             <getMainExecutor>, callback)",
        ]
    );
}

#[test]
fn java_failures_propagate() {
    for method in ["setNonce", "build", "class"] {
        let mut fake = Fake {
            fail_on: Some(method),
            ..Fake::default()
        };
        let error = build_option(&mut fake, &config(), Attempt::SignInWithGoogle, "N");
        assert_eq!(error.unwrap_err().code, ErrorCode::Bridge, "{method}");
    }
    let mut fake = Fake {
        fail_on: Some("getCredentialAsync"),
        ..Fake::default()
    };
    let request = fake.new_object("request".to_owned());
    let callback = fake.new_object("callback".to_owned());
    let error = send_request(&mut fake, request, callback).unwrap_err();
    assert_eq!(error.message, "getCredentialAsync threw");
    let mut fake = Fake {
        fail_on: Some("clearCredentialStateAsync"),
        ..Fake::default()
    };
    let callback = fake.new_object("callback".to_owned());
    let error = send_sign_out(&mut fake, callback).unwrap_err();
    assert_eq!(error.message, "clearCredentialStateAsync threw");
}

fn google_answers(fake: &mut Fake, kind: &str) {
    for (method, value) in [
        ("getType", kind),
        ("getIdToken", "jwt"),
        ("getId", "a@b.c"),
        ("getDisplayName", "Ada L"),
        ("getGivenName", "Ada"),
        ("getFamilyName", "L"),
        ("toString", "https://p/x.png"),
    ] {
        fake.answers.insert(method, JValue::Str(value.to_owned()));
    }
    fake.answers.insert("getPhoneNumber", JValue::Null);
}

#[test]
fn reads_a_google_credential() {
    for kind in [TYPE_GOOGLE_ID_TOKEN, TYPE_GOOGLE_ID_TOKEN_SIWG] {
        let mut fake = Fake::default();
        google_answers(&mut fake, kind);
        let response = fake.new_object("response".to_owned());
        let credential = read_response(&mut fake, &response).unwrap();
        assert_eq!(
            credential,
            Credential {
                id_token: "jwt".to_owned(),
                email: "a@b.c".to_owned(),
                display_name: "Ada L".to_owned(),
                given_name: "Ada".to_owned(),
                family_name: "L".to_owned(),
                picture: "https://p/x.png".to_owned(),
                phone_number: String::new(),
            }
        );
        assert_eq!(
            fake.log[..5],
            [
                "response.getCredential()",
                "<getCredential>.getType()",
                "<getCredential>.getData()",
                "class GoogleIdTokenCredential.createFrom(<getData>)",
                "<createFrom>.getProfilePictureUri()",
            ]
        );
    }
}

#[test]
fn missing_picture_is_empty() {
    let mut fake = Fake::default();
    google_answers(&mut fake, TYPE_GOOGLE_ID_TOKEN);
    fake.answers.insert("getProfilePictureUri", JValue::Null);
    let response = fake.new_object("response".to_owned());
    assert_eq!(read_response(&mut fake, &response).unwrap().picture, "");
}

#[test]
fn other_credential_types_are_refused() {
    let mut fake = Fake::default();
    google_answers(&mut fake, "androidx.credentials.TYPE_PASSWORD_CREDENTIAL");
    let response = fake.new_object("response".to_owned());
    let error = read_response(&mut fake, &response).unwrap_err();
    assert_eq!(
        error,
        SignInError::new(
            ErrorCode::UnexpectedCredential,
            "credential type androidx.credentials.TYPE_PASSWORD_CREDENTIAL"
        )
    );
}

#[test]
fn empty_token_is_invalid() {
    let mut fake = Fake::default();
    google_answers(&mut fake, TYPE_GOOGLE_ID_TOKEN);
    fake.answers.insert("getIdToken", JValue::Null);
    let response = fake.new_object("response".to_owned());
    let error = read_response(&mut fake, &response).unwrap_err();
    assert_eq!(error.code, ErrorCode::InvalidToken);
}

#[test]
fn response_needs_objects() {
    for (method, message) in [
        ("getCredential", "getCredential gave no object"),
        (
            "createFrom",
            "GoogleIdTokenCredential.createFrom gave no object",
        ),
    ] {
        let mut fake = Fake::default();
        google_answers(&mut fake, TYPE_GOOGLE_ID_TOKEN);
        fake.answers.insert(method, JValue::Int(0));
        let response = fake.new_object("response".to_owned());
        assert_eq!(
            read_response(&mut fake, &response).unwrap_err().message,
            message
        );
    }
}

#[test]
fn reads_an_exception() {
    let mut fake = Fake::default();
    fake.answers.insert(
        "getName",
        JValue::Str("androidx.credentials.exceptions.NoCredentialException".to_owned()),
    );
    fake.answers
        .insert("getMessage", JValue::Str("none here".to_owned()));
    let exception = fake.new_object("exception".to_owned());
    assert_eq!(
        read_exception(&mut fake, &exception),
        SignInError::new(ErrorCode::NoCredential, "none here")
    );
    assert_eq!(
        fake.log,
        [
            "exception.getClass()",
            "<getClass>.getName()",
            "exception.getMessage()"
        ]
    );
}

#[test]
fn unreadable_exception_is_unknown() {
    let mut fake = Fake::default();
    fake.answers.insert("getClass", JValue::Null);
    fake.answers.insert("getMessage", JValue::Null);
    let exception = fake.new_object("exception".to_owned());
    assert_eq!(
        read_exception(&mut fake, &exception),
        SignInError::new(ErrorCode::Unknown, "")
    );
}

#[test]
fn values_other_than_text_are_not_strings() {
    assert_eq!(
        JValue::<usize>::Str("a".to_owned()).into_string(),
        Some("a".to_owned())
    );
    assert_eq!(JValue::<usize>::Int(1).into_string(), None);
    assert_eq!(JValue::<usize>::Obj(0).into_string(), None);
}
