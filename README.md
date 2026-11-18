# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

Google Sign-In for Godot 4.7+ Android games, written in Rust as a
[godot-rust](https://godot-rust.github.io/) GDExtension.

It drives the Android
[Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg)
API through Godot's built-in `JavaClassWrapper` and `AndroidRuntime` plugin: no Kotlin,
no Java plugin to build.

Status: work in progress, not yet tested on a device.

## Install

1. Copy `addons/hqf_google_signin/` into your project.
2. Enable **HQF Google Sign-In** in *Project > Project Settings > Plugins*: it adds the
   Google libraries to Android exports.
3. In the Android export preset, tick *Gradle Build > Use Gradle Build* (the Google
   libraries are Maven dependencies, only the Gradle build includes them).

Prebuilt libraries (release builds, used for debug exports too): Linux x86_64 (editor)
and Android arm64, arm32, x86_64. Other
editors can still export, but the export plugin needs the extension loaded in the
editor to list the Google libraries; build from source for them (`cargo build`).

## Google Cloud setup

In the [Google Cloud console](https://console.cloud.google.com/apis/credentials) of
your project, create two OAuth clients:

- a **Web application** client: its ID goes into `web_client_id`, and your server
  checks that ID tokens are addressed to it;
- an **Android** client with your package name and the SHA-1 of the key that signs the
  APK (`keytool -list -v -keystore <your.keystore>`), one per signing key (debug and
  release). Without it, the sign-in fails with `no_credential` or `unknown`.

## Usage

```gdscript
@onready var _google: HqfGoogleSignIn = %HqfGoogleSignIn

func _ready() -> void:
	_google.web_client_id = "1234-abcd.apps.googleusercontent.com"
	_google.signed_in.connect(_on_signed_in)
	_google.sign_in_failed.connect(_on_sign_in_failed)
	if _google.is_supported():
		_google.sign_in_silently("")

func _on_sign_in_button_pressed() -> void:
	_google.sign_in_with_button("")

func _on_signed_in(account: Dictionary) -> void:
	# Send account.id_token (and account.nonce) to your server; never trust it on
	# the device alone.
	print(account.email)

func _on_sign_in_failed(code: String, message: String) -> void:
	if code != "cancelled":
		push_warning("%s: %s" % [code, message])
```

| Method | Shows |
| --- | --- |
| `sign_in(nonce)` | accounts that signed in before, every account when none did |
| `sign_in_with_account_chooser(nonce)` | every Google account of the device |
| `sign_in_with_button(nonce)` | the flow meant for a "Sign in with Google" button |
| `sign_in_silently(nonce)` | nothing when an account signed in before; else `no_credential` |
| `cancel()` | stops the running sign-in (`cancelled`) |
| `sign_out()` | forgets the account: `signed_out` or `sign_out_failed` |

An empty `nonce` is generated. `signed_in` gives `id_token`, `email`, `display_name`,
`given_name`, `family_name`, `picture`, `phone_number` and the `nonce` the request
carried. Set `hosted_domain` to only offer accounts of a Google Workspace domain, and
`request_verified_phone_number` to ask for a verified phone number.

### Error codes

| Code | Meaning |
| --- | --- |
| `cancelled` | the player closed the sheet, or the game called `cancel()` |
| `no_credential` | no Google account matches the request on this device |
| `interrupted` | the request was interrupted; it may be retried |
| `provider_configuration` | no credential provider set up for Google accounts |
| `unsupported` | the device does not support the Credential Manager API |
| `unknown` | the provider failed without saying why |
| `custom` | the provider raised an error of its own kind |
| `unexpected_credential` | the answer is not a Google ID token |
| `invalid_token` | the Google ID token could not be read |
| `invalid_config` | `web_client_id` (or another setting) is not usable |
| `busy` | a sign-in or sign-out is already running |
| `unsupported_platform` | not Android, or the Godot Android runtime is missing |
| `bridge` | a call into the Android API failed |

### On the server

The ID token proves nothing until your server verifies it: signature (Google's public
keys), `aud` = your Web client ID, `iss` = `accounts.google.com` or
`https://accounts.google.com`, `exp` in the future, and `nonce` = the one the game
sent. See [Verify the Google ID token on your server side](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token).

## Build

`zsh tools/build.zsh linux|android [debug|release]` builds the library into
`addons/hqf_google_signin/bin/` (Android: the NDK of the Android SDK).

## Quality

Checked once a day by the CI badge above (never on each push): `cargo fmt`, `clippy`
with every warning as an error (`all`, `pedantic`, `nursery`), unit, property
(`proptest`) and documentation tests, mutation testing with `cargo-mutants`, and
`cargo-deny` for licenses and security advisories. The ID token reader is fuzzed with
`cargo-fuzz`; its corpus lives in `fuzz/corpus/`.

## License

MIT, see [LICENSE](LICENSE).
