# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

Google Sign-In untuk game Android buatan Godot 4.7+, ditulis dengan Rust sebagai GDExtension [godot-rust](https://godot-rust.github.io/).

Addon ini menjalankan API [Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg) Android melalui `JavaClassWrapper` dan plugin `AndroidRuntime` bawaan Godot: tanpa Kotlin, tanpa plugin Java yang perlu di-build.

## Instalasi

1. Salin `addons/hqf_google_signin/` ke proyek Anda.
2. Aktifkan **HQF Google Sign-In** di *Project > Project Settings > Plugins*: plugin ini menambahkan library Google ke ekspor Android.
3. Di preset ekspor Android, centang *Gradle Build > Use Gradle Build* (library Google adalah dependensi Maven, dan hanya build Gradle yang menyertakannya).

Library siap pakai (build release, juga dipakai untuk ekspor debug): Linux x86_64 (editor) serta Android arm64, arm32, dan x86_64. Editor di platform lain tetap bisa mengekspor, tetapi plugin ekspor membutuhkan ekstensi yang sudah dimuat di editor untuk menampilkan daftar library Google; di platform tersebut, build dari kode sumber (`cargo build`).

## Penyiapan Google Cloud

Di [konsol Google Cloud](https://console.cloud.google.com/apis/credentials) proyek Anda, buat dua klien OAuth:

- klien **Aplikasi web**: ID-nya dimasukkan ke `web_client_id`, dan server Anda memeriksa bahwa token ID memang ditujukan untuk klien ini;
- klien **Android** dengan nama paket Anda dan sidik jari SHA-1 dari kunci yang menandatangani APK (`keytool -list -v -keystore <your.keystore>`), satu untuk setiap kunci penandatanganan (debug dan release). Tanpa klien ini, login gagal dengan `no_credential` atau `unknown`.

## Penggunaan

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

| Metode | Menampilkan |
| --- | --- |
| `sign_in(nonce)` | akun yang pernah login sebelumnya, atau semua akun jika belum ada yang pernah login |
| `sign_in_with_account_chooser(nonce)` | semua akun Google di perangkat |
| `sign_in_with_button(nonce)` | alur yang dirancang untuk tombol “Login dengan Google” |
| `sign_in_silently(nonce)` | tidak ada jika ada akun yang pernah login sebelumnya; jika tidak, `no_credential` |
| `cancel()` | menghentikan proses login yang sedang berjalan (`cancelled`) |
| `sign_out()` | melupakan akun: `signed_out` atau `sign_out_failed` |

Jika `nonce` kosong, nilainya dibuat secara otomatis. `signed_in` memberikan `id_token`, `email`, `display_name`, `given_name`, `family_name`, `picture`, `phone_number`, dan `nonce` yang dikirim bersama permintaan. Isi `hosted_domain` agar hanya akun dari domain Google Workspace tertentu yang ditawarkan, dan `request_verified_phone_number` untuk meminta nomor telepon yang terverifikasi.

### Kode Error

| Kode | Arti |
| --- | --- |
| `cancelled` | pemain menutup panel, atau game memanggil `cancel()` |
| `no_credential` | tidak ada akun Google di perangkat ini yang cocok dengan permintaan |
| `interrupted` | permintaan terputus; bisa dicoba lagi |
| `provider_configuration` | belum ada penyedia kredensial yang disiapkan untuk akun Google |
| `unsupported` | perangkat tidak mendukung API Credential Manager |
| `unknown` | penyedia gagal tanpa menyebutkan alasannya |
| `custom` | penyedia memunculkan error dengan jenisnya sendiri |
| `unexpected_credential` | respons yang diterima bukan token ID Google |
| `invalid_token` | token ID Google tidak dapat dibaca |
| `invalid_config` | `web_client_id` (atau setelan lain) tidak dapat digunakan |
| `busy` | proses login atau logout sedang berjalan |
| `unsupported_platform` | platformnya bukan Android, atau runtime Android Godot tidak tersedia |
| `bridge` | panggilan ke API Android gagal |

### Di Sisi Server

Token ID tidak membuktikan apa pun sebelum diverifikasi oleh server Anda: tanda tangan (kunci publik Google), `aud` = ID klien web Anda, `iss` = `accounts.google.com` atau `https://accounts.google.com`, `exp` berada di masa depan, dan `nonce` = nilai yang dikirim oleh game. Lihat [Memverifikasi token ID Google di sisi server](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token).

## Build

`zsh tools/build.zsh linux|android [debug|release]` mem-build library ke `addons/hqf_google_signin/bin/` (Android: menggunakan NDK dari Android SDK).

## Kualitas

Proyek ini diperiksa sekali sehari oleh CI pada badge di atas (tidak pernah di setiap push): `cargo fmt`, `clippy` dengan setiap peringatan diperlakukan sebagai error (`all`, `pedantic`, `nursery`), pengujian unit, properti (`proptest`), dan dokumentasi, pengujian mutasi dengan `cargo-mutants`, serta `cargo-deny` untuk lisensi dan peringatan keamanan. Pembaca token ID diuji dengan fuzzing menggunakan `cargo-fuzz`; korpusnya ada di `fuzz/corpus/`.

## Lisensi

MIT, lihat [LICENSE](LICENSE).
