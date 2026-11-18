# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

یہ Godot 4.7+ کے Android گیمز کے لیے Google Sign-In ہے، جو Rust میں [godot-rust](https://godot-rust.github.io/) پر مبنی GDExtension کی صورت میں لکھا گیا ہے۔

یہ Godot میں پہلے سے موجود `JavaClassWrapper` class اور `AndroidRuntime` plugin کے ذریعے Android کی [Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg) API استعمال کرتا ہے: نہ Kotlin کی ضرورت ہے، نہ کوئی Java plugin بنانا پڑتا ہے۔

## انسٹالیشن

1. اپنے پروجیکٹ میں `addons/hqf_google_signin/` فولڈر کاپی کریں۔
2. مینو *Project > Project Settings > Plugins* میں **HQF Google Sign-In** کو فعال کریں: یہ Android export میں Google لائبریریاں شامل کر دیتا ہے۔
3. اپنے Android export preset میں *Gradle Build > Use Gradle Build* پر ٹک کریں (Google لائبریریاں Maven dependencies ہیں، اور انہیں صرف Gradle build ہی شامل کرتا ہے)۔

پہلے سے بنی ہوئی لائبریریاں (release builds، جو debug export کے لیے بھی استعمال ہوتی ہیں): Linux x86_64 (ایڈیٹر) اور Android arm64، arm32، x86_64۔ دوسرے ایڈیٹرز سے بھی export کیا جا سکتا ہے، لیکن Google لائبریریوں کی فہرست بنانے کے لیے export plugin کو ضرورت ہوتی ہے کہ extension ایڈیٹر میں لوڈ ہو؛ ایسے ایڈیٹرز کے لیے source سے build کریں (`cargo build`)۔

## سیٹ اپ: Google Cloud

اپنے پروجیکٹ کے [Google Cloud console](https://console.cloud.google.com/apis/credentials) میں دو OAuth clients بنائیں:

- ایک **Web application** client: اس کی ID `web_client_id` میں جاتی ہے، اور آپ کا سرور جانچتا ہے کہ ID tokens اسی client کے لیے جاری ہوئے ہیں؛
- ایک **Android** client، جس میں آپ کا package name اور APK پر sign کرنے والی key کا SHA-1 ہو (`keytool -list -v -keystore <your.keystore>`)؛ ہر signing key کے لیے ایک (debug اور release)۔ اس کے بغیر sign-in ناکام ہو جاتا ہے، اور ایرر `no_credential` یا `unknown` آتا ہے۔

## استعمال

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

| میتھڈ | کیا دکھاتا ہے |
| --- | --- |
| `sign_in(nonce)` | وہ اکاؤنٹس جن سے پہلے sign in کیا گیا تھا؛ اگر ایسا کوئی نہیں، تو تمام اکاؤنٹس |
| `sign_in_with_account_chooser(nonce)` | ڈیوائس کے تمام Google اکاؤنٹس |
| `sign_in_with_button(nonce)` | وہ flow جو “Sign in with Google” بٹن کے لیے بنایا گیا ہے |
| `sign_in_silently(nonce)` | اگر کسی اکاؤنٹ سے پہلے sign in ہو چکا ہو تو کچھ نہیں؛ ورنہ `no_credential` |
| `cancel()` | جاری sign-in کو روک دیتا ہے (`cancelled`) |
| `sign_out()` | اکاؤنٹ کو بھول جاتا ہے: `signed_out` یا `sign_out_failed` |

اگر `nonce` خالی ہو تو وہ خود بخود بنا دیا جاتا ہے۔ `signed_in` سے `id_token`، `email`، `display_name`، `given_name`، `family_name`، `picture`، `phone_number` اور درخواست کے ساتھ بھیجا گیا `nonce` ملتا ہے۔ صرف کسی Google Workspace domain کے اکاؤنٹس دکھانے کے لیے `hosted_domain` سیٹ کریں، اور تصدیق شدہ فون نمبر مانگنے کے لیے `request_verified_phone_number` سیٹ کریں۔

### ایرر کوڈز

| کوڈ | مطلب |
| --- | --- |
| `cancelled` | کھلاڑی نے sign-in sheet بند کر دی، یا گیم نے `cancel()` کال کیا |
| `no_credential` | اس ڈیوائس پر کوئی Google اکاؤنٹ درخواست سے میل نہیں کھاتا |
| `interrupted` | درخواست درمیان میں رک گئی؛ دوبارہ کوشش کی جا سکتی ہے |
| `provider_configuration` | Google اکاؤنٹس کے لیے کوئی credential provider سیٹ نہیں کیا گیا |
| `unsupported` | یہ ڈیوائس Credential Manager API کو سپورٹ نہیں کرتی |
| `unknown` | وجہ بتائے بغیر provider ناکام ہو گیا |
| `custom` | اپنی ہی نوعیت کی ایک ایرر، جو provider نے دی |
| `unexpected_credential` | جواب Google ID token نہیں ہے |
| `invalid_token` | دیا گیا Google ID token پڑھا نہیں جا سکا |
| `invalid_config` | دی گئی `web_client_id` (یا کوئی اور سیٹنگ) قابلِ استعمال نہیں |
| `busy` | کوئی sign-in یا sign-out پہلے سے جاری ہے |
| `unsupported_platform` | پلیٹ فارم Android نہیں، یا Godot Android runtime موجود نہیں |
| `bridge` | Android API کو کی گئی ایک کال ناکام ہو گئی |

### سرور پر

جب تک آپ کا سرور ID token کی تصدیق نہ کرے، وہ کچھ ثابت نہیں کرتا۔ یہ چیزیں جانچیں: signature (Google کی public keys)، `aud` = آپ کی Web client ID، `iss` = `accounts.google.com` یا `https://accounts.google.com`، `exp` مستقبل کا وقت ہو، اور `nonce` = وہی جو گیم نے بھیجا تھا۔ دیکھیں: [اپنے سرور پر Google ID token کی تصدیق کریں](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token)۔

## بلڈ

کمانڈ `zsh tools/build.zsh linux|android [debug|release]` لائبریری کو `addons/hqf_google_signin/bin/` میں build کرتی ہے (Android کے لیے: Android SDK والا NDK)۔

## کوالٹی

اوپر والے badge کا CI اسے دن میں ایک بار جانچتا ہے (ہر push پر کبھی نہیں): `cargo fmt`؛ `clippy`، جس میں ہر warning کو ایرر مانا جاتا ہے (`all`، `pedantic`، `nursery`)؛ unit، property (`proptest`) اور documentation tests؛ `cargo-mutants` کے ذریعے mutation testing؛ اور licenses اور security advisories کے لیے `cargo-deny`۔ ID token reader کو `cargo-fuzz` سے fuzz کیا جاتا ہے؛ اس کا corpus `fuzz/corpus/` میں ہے۔

## لائسنس

لائسنس: MIT، دیکھیں [LICENSE](LICENSE)۔
