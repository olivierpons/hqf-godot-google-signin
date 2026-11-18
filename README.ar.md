# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

تسجيل الدخول عبر Google (Google Sign-In) لألعاب Android المبنية على Godot 4.7+، مكتوب بلغة Rust في صورة امتداد GDExtension مبني على [godot-rust](https://godot-rust.github.io/).

يستدعي واجهة [Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg) البرمجية (API) في Android عبر الصنف `JavaClassWrapper` والإضافة `AndroidRuntime`، وكلاهما مدمج في Godot: لا حاجة إلى Kotlin، ولا إلى بناء أي إضافة Java.

## التثبيت

1. انسخ المجلد `addons/hqf_google_signin/` إلى مشروعك.
2. فعّل الإضافة **HQF Google Sign-In** من *Project > Project Settings > Plugins*: فهي تضيف مكتبات Google إلى عمليات تصدير Android.
3. في الإعداد المسبق لتصدير Android (export preset)، فعّل الخيار *Gradle Build > Use Gradle Build* (مكتبات Google اعتماديات Maven، ولا يضمّنها إلا بناء Gradle).

المكتبات المبنية مسبقًا (بنسخة release، وتُستخدم أيضًا في عمليات تصدير debug): Linux x86_64 (للمحرر)، و Android arm64 و arm32 و x86_64. يمكن للمحررات الأخرى التصدير أيضًا، لكن إضافة التصدير تحتاج إلى أن يكون الامتداد محمَّلًا في المحرر كي تعرف مكتبات Google المطلوبة؛ لهذه المحررات، ابنِ المكتبة من الشيفرة المصدرية (`cargo build`).

## إعداد Google Cloud

في [وحدة تحكم Google Cloud](https://console.cloud.google.com/apis/credentials) الخاصة بمشروعك، أنشئ عميلَي OAuth:

- عميل من نوع **Web application**: يوضع معرّفه في `web_client_id`، ويتحقق خادمك من أن رموز الهوية (ID tokens) صادرة لهذا العميل؛
- عميل من نوع **Android** يتضمن اسم الحزمة الخاص بك وبصمة SHA-1 للمفتاح الذي يوقّع ملف APK (`keytool -list -v -keystore <your.keystore>`)، بمعدل عميل واحد لكل مفتاح توقيع (debug و release). من دون هذا العميل، يفشل تسجيل الدخول بالكود `no_credential` أو `unknown`.

## الاستخدام

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

| الدالة | ما تعرضه |
| --- | --- |
| `sign_in(nonce)` | الحسابات التي سُجّل الدخول بها من قبل، أو جميع الحسابات إن لم يُسجَّل الدخول بأي منها |
| `sign_in_with_account_chooser(nonce)` | جميع حسابات Google الموجودة على الجهاز |
| `sign_in_with_button(nonce)` | المسار المخصص لزر «Sign in with Google» |
| `sign_in_silently(nonce)` | لا شيء إذا سبق تسجيل الدخول بأحد الحسابات؛ وإلا فالكود `no_credential` |
| `cancel()` | توقف عملية تسجيل الدخول الجارية (`cancelled`) |
| `sign_out()` | تنسى الحساب: `signed_out` أو `sign_out_failed` |

إذا كانت قيمة `nonce` فارغة، تُولَّد تلقائيًا. تعيد الإشارة `signed_in` القيم `id_token` و `email` و `display_name` و `given_name` و `family_name` و `picture` و `phone_number`، إضافةً إلى قيمة `nonce` التي أُرسلت مع الطلب. اضبط `hosted_domain` كي لا تُعرض إلا حسابات نطاق Google Workspace معيّن، واضبط `request_verified_phone_number` لطلب رقم هاتف موثَّق.

### أكواد الأخطاء

| الكود | المعنى |
| --- | --- |
| `cancelled` | أغلق اللاعب نافذة تسجيل الدخول، أو استدعت اللعبة `cancel()` |
| `no_credential` | لا يوجد على هذا الجهاز حساب Google يطابق الطلب |
| `interrupted` | انقطع الطلب؛ ويمكن إعادة المحاولة |
| `provider_configuration` | لم يُضبط أي موفّر لبيانات الاعتماد لحسابات Google |
| `unsupported` | الجهاز لا يدعم واجهة Credential Manager البرمجية |
| `unknown` | فشل الموفّر دون أن يذكر السبب |
| `custom` | أطلق الموفّر خطأً من نوع خاص به |
| `unexpected_credential` | الاستجابة ليست رمز هوية من Google |
| `invalid_token` | تعذّرت قراءة رمز الهوية من Google |
| `invalid_config` | القيمة `web_client_id` (أو إعداد آخر) غير صالحة للاستخدام |
| `busy` | هناك عملية تسجيل دخول أو تسجيل خروج جارية بالفعل |
| `unsupported_platform` | المنصة ليست Android، أو بيئة تشغيل Godot الخاصة بـ Android غير موجودة |
| `bridge` | فشل استدعاء موجّه إلى واجهة Android البرمجية |

### على الخادم

لا يُثبت رمز الهوية شيئًا ما لم يتحقق منه خادمك: التوقيع (بالمفاتيح العامة لـ Google)، و `aud` = معرّف عميل Web الخاص بك، و `iss` = `accounts.google.com` أو `https://accounts.google.com`، و `exp` في المستقبل، و `nonce` = القيمة نفسها التي أرسلتها اللعبة. راجع [التحقق من رمز هوية Google على جانب الخادم](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token).

## البناء

يبني الأمر `zsh tools/build.zsh linux|android [debug|release]` المكتبة في المجلد `addons/hqf_google_signin/bin/` (في Android: باستخدام NDK الموجود في Android SDK).

## الجودة

يفحص الـ CI الذي تشير إليه الشارة أعلاه المشروعَ مرة واحدة يوميًا (لا عند كل push): `cargo fmt`؛ و `clippy` مع معاملة كل تحذير كخطأ (`all` و `pedantic` و `nursery`)؛ واختبارات الوحدات، واختبارات الخصائص (`proptest`)، واختبارات التوثيق؛ واختبار الطفرات (mutation testing) باستخدام `cargo-mutants`؛ و `cargo-deny` لفحص التراخيص والتنبيهات الأمنية. ويخضع قارئ رموز الهوية لاختبار التشويش (fuzzing) باستخدام `cargo-fuzz`، وتوجد مجموعة مدخلاته (corpus) في `fuzz/corpus/`.

## الترخيص

مرخّص بموجب MIT، راجع [LICENSE](LICENSE).
