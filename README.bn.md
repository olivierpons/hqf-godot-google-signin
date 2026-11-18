# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

Godot 4.7+ দিয়ে তৈরি Android গেমের জন্য Google Sign-In, Rust-এ লেখা একটি
[godot-rust](https://godot-rust.github.io/) ভিত্তিক GDExtension।

এটি Godot-এর built-in `JavaClassWrapper` class আর `AndroidRuntime` plugin-এর মাধ্যমে Android-এর
[Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg)
API ব্যবহার করে: Kotlin লাগে না, কোনো Java plugin build করারও দরকার নেই।

## ইনস্টলেশন

1. `addons/hqf_google_signin/` ফোল্ডারটি আপনার প্রজেক্টে কপি করুন।
2. *Project > Project Settings > Plugins* থেকে **HQF Google Sign-In** চালু করুন: এটি Android export-এ Google লাইব্রেরিগুলো যোগ করে।
3. Android export preset-এ *Gradle Build > Use Gradle Build*-এ টিক দিন (Google লাইব্রেরিগুলো Maven dependency, আর শুধু Gradle build-ই সেগুলো অন্তর্ভুক্ত করে)।

আগে থেকে build করা লাইব্রেরি (release build, debug export-এও এগুলোই ব্যবহৃত হয়): Linux x86_64 (এডিটর) এবং Android arm64, arm32, x86_64। অন্য এডিটর থেকেও export করা যায়, তবে Google লাইব্রেরিগুলোর তালিকা পেতে export plugin-এর জন্য এডিটরে extension-টি লোড থাকা দরকার; সেসব এডিটরের জন্য source থেকে build করুন (`cargo build`)।

## Google Cloud সেটআপ

আপনার প্রজেক্টের [Google Cloud console](https://console.cloud.google.com/apis/credentials)-এ দুটি OAuth client তৈরি করুন:

- একটি **Web application** client: এর ID বসবে `web_client_id`-তে, আর আপনার সার্ভার যাচাই করে যে ID token-গুলো এই client-এর জন্যই ইস্যু করা হয়েছে;
- একটি **Android** client, যাতে থাকবে আপনার package name এবং যে key দিয়ে APK sign করা হয় তার SHA-1 (`keytool -list -v -keystore <your.keystore>`); প্রতিটি signing key-র জন্য একটি করে (debug ও release)। এটি না থাকলে sign-in `no_credential` বা `unknown` এরর দিয়ে ব্যর্থ হয়।

## ব্যবহার

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

| মেথড | কী দেখায় |
| --- | --- |
| `sign_in(nonce)` | যেসব অ্যাকাউন্ট দিয়ে আগে sign in করা হয়েছে; এমন কোনোটি না থাকলে সব অ্যাকাউন্ট |
| `sign_in_with_account_chooser(nonce)` | ডিভাইসের সব Google অ্যাকাউন্ট |
| `sign_in_with_button(nonce)` | “Sign in with Google” বোতামের জন্য তৈরি flow |
| `sign_in_silently(nonce)` | কোনো অ্যাকাউন্ট দিয়ে আগে sign in করা থাকলে কিছুই না; নইলে `no_credential` |
| `cancel()` | চলমান sign-in থামায় (`cancelled`) |
| `sign_out()` | অ্যাকাউন্টটি ভুলে যায়: `signed_out` বা `sign_out_failed` |

`nonce` খালি থাকলে সেটি নিজে থেকেই তৈরি হয়। `signed_in` থেকে পাওয়া যায় `id_token`, `email`, `display_name`, `given_name`, `family_name`, `picture`, `phone_number` এবং রিকোয়েস্টের সঙ্গে পাঠানো `nonce`। শুধু কোনো Google Workspace domain-এর অ্যাকাউন্ট দেখাতে `hosted_domain` সেট করুন, আর যাচাই করা ফোন নম্বর চাইতে `request_verified_phone_number` সেট করুন।

### এরর কোড

| কোড | অর্থ |
| --- | --- |
| `cancelled` | খেলোয়াড় sign-in sheet বন্ধ করেছে, অথবা গেম `cancel()` call করেছে |
| `no_credential` | এই ডিভাইসে কোনো Google অ্যাকাউন্ট রিকোয়েস্টের সঙ্গে মেলে না |
| `interrupted` | রিকোয়েস্টটি মাঝপথে থেমে গেছে; আবার চেষ্টা করা যেতে পারে |
| `provider_configuration` | Google অ্যাকাউন্টের জন্য কোনো credential provider সেট করা নেই |
| `unsupported` | ডিভাইসটি Credential Manager API সমর্থন করে না |
| `unknown` | provider কোনো কারণ না জানিয়েই ব্যর্থ হয়েছে |
| `custom` | provider তার নিজস্ব ধরনের একটি এরর দিয়েছে |
| `unexpected_credential` | উত্তরটি Google ID token নয় |
| `invalid_token` | Google ID token পড়া যায়নি |
| `invalid_config` | `web_client_id` (বা অন্য কোনো সেটিং) ব্যবহারযোগ্য নয় |
| `busy` | একটি sign-in বা sign-out আগে থেকেই চলছে |
| `unsupported_platform` | প্ল্যাটফর্মটি Android নয়, অথবা Godot Android runtime নেই |
| `bridge` | Android API-তে করা একটি call ব্যর্থ হয়েছে |

### সার্ভারে

আপনার সার্ভার যাচাই না করা পর্যন্ত ID token কিছুই প্রমাণ করে না। যা যাচাই করতে হবে: signature (Google-এর public key), `aud` = আপনার Web client ID, `iss` = `accounts.google.com` বা `https://accounts.google.com`, `exp` ভবিষ্যতের সময়, এবং `nonce` = গেম যেটি পাঠিয়েছিল ঠিক সেটিই। দেখুন: [আপনার সার্ভারে Google ID token যাচাই করুন](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token)।

## বিল্ড

`zsh tools/build.zsh linux|android [debug|release]` লাইব্রেরিটি
`addons/hqf_google_signin/bin/`-এ build করে (Android-এর জন্য: Android SDK-র NDK)।

## কোয়ালিটি

উপরের badge-এর CI দিনে একবার এটি যাচাই করে (প্রতিটি push-এ কখনও নয়): `cargo fmt`; `clippy`, যেখানে প্রতিটি warning-কে এরর ধরা হয় (`all`, `pedantic`, `nursery`); unit, property (`proptest`) ও documentation test; `cargo-mutants` দিয়ে mutation testing; এবং license ও security advisory যাচাইয়ের জন্য `cargo-deny`। ID token reader-কে `cargo-fuzz` দিয়ে fuzz করা হয়; এর corpus রয়েছে `fuzz/corpus/`-এ।

## লাইসেন্স

MIT, দেখুন [LICENSE](LICENSE)।
