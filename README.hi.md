# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

Godot 4.7+ के Android गेम्स के लिए Google Sign-In, जिसे Rust में
[godot-rust](https://godot-rust.github.io/) पर आधारित GDExtension के रूप में लिखा गया है।

यह Godot की built-in `JavaClassWrapper` class और `AndroidRuntime` plugin के ज़रिए Android की
[Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg)
API इस्तेमाल करता है: न Kotlin चाहिए, न कोई Java plugin build करना पड़ता है।

स्थिति: काम जारी है, अभी तक किसी डिवाइस पर टेस्ट नहीं किया गया।

## इंस्टॉलेशन

1. `addons/hqf_google_signin/` को अपने प्रोजेक्ट में कॉपी करें।
2. *Project > Project Settings > Plugins* में **HQF Google Sign-In** को enable करें: यह Android export में Google लाइब्रेरियाँ जोड़ देता है।
3. Android export preset में *Gradle Build > Use Gradle Build* पर टिक करें (Google लाइब्रेरियाँ Maven dependencies हैं, और उन्हें सिर्फ़ Gradle build ही शामिल करता है)।

पहले से बनी लाइब्रेरियाँ (release builds, जो debug export के लिए भी इस्तेमाल होती हैं): Linux x86_64 (एडिटर) और Android arm64, arm32, x86_64। दूसरे एडिटर से भी export किया जा सकता है, लेकिन Google लाइब्रेरियों की सूची बनाने के लिए export plugin को एडिटर में extension का लोड होना ज़रूरी है; ऐसे एडिटर के लिए source से build करें (`cargo build`)।

## Google Cloud सेटअप

अपने प्रोजेक्ट के [Google Cloud console](https://console.cloud.google.com/apis/credentials) में दो OAuth clients बनाएँ:

- एक **Web application** client: इसकी ID `web_client_id` में जाती है, और आपका सर्वर जाँचता है कि ID tokens इसी client के लिए जारी हुए हैं;
- एक **Android** client, जिसमें आपका package name और APK को sign करने वाली key का SHA-1 हो (`keytool -list -v -keystore <your.keystore>`); हर signing key के लिए एक (debug और release)। इसके बिना sign-in `no_credential` या `unknown` एरर के साथ fail हो जाता है।

## इस्तेमाल

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

| मेथड | क्या दिखाता है |
| --- | --- |
| `sign_in(nonce)` | वे अकाउंट जिनसे पहले sign in किया गया था; अगर ऐसा कोई नहीं, तो सभी अकाउंट |
| `sign_in_with_account_chooser(nonce)` | डिवाइस के सभी Google अकाउंट |
| `sign_in_with_button(nonce)` | “Sign in with Google” बटन के लिए बना flow |
| `sign_in_silently(nonce)` | अगर किसी अकाउंट से पहले sign in हुआ था, तो कुछ नहीं; वरना `no_credential` |
| `cancel()` | चल रहे sign-in को रोकता है (`cancelled`) |
| `sign_out()` | अकाउंट को भूल जाता है: `signed_out` या `sign_out_failed` |

अगर `nonce` खाली हो, तो वह अपने-आप generate हो जाता है। `signed_in` से `id_token`, `email`, `display_name`, `given_name`, `family_name`, `picture`, `phone_number` और रिक्वेस्ट के साथ भेजा गया `nonce` मिलता है। सिर्फ़ किसी Google Workspace domain के अकाउंट दिखाने के लिए `hosted_domain` सेट करें, और verified फ़ोन नंबर माँगने के लिए `request_verified_phone_number` सेट करें।

### एरर कोड

| कोड | मतलब |
| --- | --- |
| `cancelled` | खिलाड़ी ने sign-in sheet बंद कर दी, या गेम ने `cancel()` call किया |
| `no_credential` | इस डिवाइस पर कोई भी Google अकाउंट रिक्वेस्ट से मेल नहीं खाता |
| `interrupted` | रिक्वेस्ट बीच में रुक गई; इसे दोबारा आज़माया जा सकता है |
| `provider_configuration` | Google अकाउंट के लिए कोई credential provider सेट नहीं है |
| `unsupported` | डिवाइस Credential Manager API को सपोर्ट नहीं करता |
| `unknown` | provider बिना कोई कारण बताए fail हो गया |
| `custom` | provider ने अपनी ही किस्म की कोई एरर दी |
| `unexpected_credential` | जवाब Google ID token नहीं है |
| `invalid_token` | Google ID token पढ़ा नहीं जा सका |
| `invalid_config` | `web_client_id` (या कोई दूसरी सेटिंग) इस्तेमाल करने लायक नहीं है |
| `busy` | कोई sign-in या sign-out पहले से चल रहा है |
| `unsupported_platform` | प्लैटफ़ॉर्म Android नहीं है, या Godot Android runtime मौजूद नहीं है |
| `bridge` | Android API को की गई कोई call fail हो गई |

### सर्वर पर

जब तक आपका सर्वर ID token को verify नहीं करता, तब तक वह कुछ साबित नहीं करता। ये चीज़ें जाँचें: signature (Google की public keys), `aud` = आपकी Web client ID, `iss` = `accounts.google.com` या `https://accounts.google.com`, `exp` भविष्य का समय हो, और `nonce` = वही जो गेम ने भेजा था। देखें: [अपने सर्वर पर Google ID token verify करें](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token)।

## बिल्ड

`zsh tools/build.zsh linux|android [debug|release]` लाइब्रेरी को
`addons/hqf_google_signin/bin/` में build करता है (Android के लिए: Android SDK वाला NDK)।

## क्वालिटी

ऊपर दिए badge वाला CI इसे दिन में एक बार जाँचता है (हर push पर कभी नहीं): `cargo fmt`; `clippy`, जिसमें हर warning को एरर माना जाता है (`all`, `pedantic`, `nursery`); unit, property (`proptest`) और documentation tests; `cargo-mutants` से mutation testing; और licenses व security advisories के लिए `cargo-deny`। ID token reader को `cargo-fuzz` से fuzz किया जाता है; इसका corpus `fuzz/corpus/` में है।

## लाइसेंस

MIT, देखें [LICENSE](LICENSE)।
