# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

Google Sign-In para juegos Android hechos con Godot 4.7+, escrito en Rust como GDExtension de [godot-rust](https://godot-rust.github.io/).

Controla la API [Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg) de Android a través de `JavaClassWrapper` y del plugin `AndroidRuntime` integrados en Godot: sin Kotlin y sin ningún plugin Java que compilar.

## Instalación

1. Copia `addons/hqf_google_signin/` en tu proyecto.
2. Activa **HQF Google Sign-In** en *Project > Project Settings > Plugins*: el plugin añade las bibliotecas de Google a las exportaciones de Android.
3. En el preset de exportación de Android, marca *Gradle Build > Use Gradle Build* (las bibliotecas de Google son dependencias de Maven y solo el build de Gradle las incluye).

Bibliotecas precompiladas (builds de release, que también se usan en las exportaciones de debug): Linux x86_64 (editor) y Android arm64, arm32, x86_64. Los editores de otras plataformas también pueden exportar, pero el plugin de exportación necesita la extensión cargada en el editor para listar las bibliotecas de Google; en esas plataformas, compila desde el código fuente (`cargo build`).

## Configuración de Google Cloud

En la [consola de Google Cloud](https://console.cloud.google.com/apis/credentials) de tu proyecto, crea dos clientes OAuth:

- un cliente **Aplicación web**: su ID va en `web_client_id`, y tu servidor comprueba que los tokens de ID van dirigidos a él;
- un cliente **Android** con el nombre de tu paquete y la huella SHA-1 de la clave que firma el APK (`keytool -list -v -keystore <your.keystore>`), uno por cada clave de firma (debug y release). Sin él, el inicio de sesión falla con `no_credential` o `unknown`.

## Uso

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

| Método | Muestra |
| --- | --- |
| `sign_in(nonce)` | las cuentas que ya iniciaron sesión antes o, si no hay ninguna, todas las cuentas |
| `sign_in_with_account_chooser(nonce)` | todas las cuentas de Google del dispositivo |
| `sign_in_with_button(nonce)` | el flujo pensado para un botón «Iniciar sesión con Google» |
| `sign_in_silently(nonce)` | nada si una cuenta ya inició sesión antes; si no, `no_credential` |
| `cancel()` | detiene el inicio de sesión en curso (`cancelled`) |
| `sign_out()` | olvida la cuenta: `signed_out` o `sign_out_failed` |

Si `nonce` está vacío, se genera automáticamente. `signed_in` devuelve `id_token`, `email`, `display_name`, `given_name`, `family_name`, `picture`, `phone_number` y el `nonce` que se envió con la solicitud. Configura `hosted_domain` para ofrecer solo las cuentas de un dominio de Google Workspace, y `request_verified_phone_number` para pedir un número de teléfono verificado.

### Códigos de error

| Código | Significado |
| --- | --- |
| `cancelled` | el jugador cerró el panel o el juego llamó a `cancel()` |
| `no_credential` | ninguna cuenta de Google de este dispositivo coincide con la solicitud |
| `interrupted` | la solicitud se interrumpió; se puede reintentar |
| `provider_configuration` | no hay ningún proveedor de credenciales configurado para las cuentas de Google |
| `unsupported` | el dispositivo no es compatible con la API Credential Manager |
| `unknown` | el proveedor falló sin indicar el motivo |
| `custom` | el proveedor devolvió un error de un tipo propio |
| `unexpected_credential` | la respuesta no es un token de ID de Google |
| `invalid_token` | no se pudo leer el token de ID de Google |
| `invalid_config` | `web_client_id` (u otro ajuste) no se puede usar |
| `busy` | ya hay un inicio o un cierre de sesión en curso |
| `unsupported_platform` | la plataforma no es Android, o falta el runtime de Android de Godot |
| `bridge` | falló una llamada a la API de Android |

### En el servidor

El token de ID no demuestra nada hasta que tu servidor lo verifica: la firma (claves públicas de Google), `aud` = el ID de tu cliente web, `iss` = `accounts.google.com` o `https://accounts.google.com`, `exp` en el futuro y `nonce` = el que envió el juego. Consulta [Verificar el token de ID de Google en tu servidor](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token).

## Compilación

`zsh tools/build.zsh linux|android [debug|release]` compila la biblioteca en `addons/hqf_google_signin/bin/` (Android: con el NDK del SDK de Android).

## Calidad

La CI de la insignia de arriba revisa el proyecto una vez al día (nunca en cada push): `cargo fmt`, `clippy` con todas las advertencias tratadas como errores (`all`, `pedantic`, `nursery`), pruebas unitarias, de propiedades (`proptest`) y de documentación, pruebas de mutación con `cargo-mutants`, y `cargo-deny` para licencias y avisos de seguridad. El lector de tokens de ID se somete a fuzzing con `cargo-fuzz`; su corpus está en `fuzz/corpus/`.

## Licencia

MIT, consulta [LICENSE](LICENSE).
