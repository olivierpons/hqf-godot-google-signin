# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

Google Sign-In pour les jeux Android sous Godot 4.7+, écrit en Rust sous forme de GDExtension [godot-rust](https://godot-rust.github.io/).

Il pilote l’API Android [Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg) au moyen de `JavaClassWrapper` et du plugin `AndroidRuntime` intégrés à Godot : pas de Kotlin, pas de plugin Java à compiler.

État : en cours de développement, pas encore testé sur un appareil.

## Installation

1. Copiez `addons/hqf_google_signin/` dans votre projet.
2. Activez **HQF Google Sign-In** dans *Project > Project Settings > Plugins* : le plugin ajoute les bibliothèques Google aux exports Android.
3. Dans le préréglage d’export Android, cochez *Gradle Build > Use Gradle Build* (les bibliothèques Google sont des dépendances Maven, et seul le build Gradle les intègre).

Bibliothèques précompilées (builds release, utilisées aussi pour les exports debug) : Linux x86_64 (éditeur) et Android arm64, arm32, x86_64. Les éditeurs des autres plateformes peuvent quand même exporter, mais le plugin d’export a besoin que l’extension soit chargée dans l’éditeur pour lister les bibliothèques Google ; sur ces plateformes, compilez depuis les sources (`cargo build`).

## Configuration de Google Cloud

Dans la [console Google Cloud](https://console.cloud.google.com/apis/credentials) de votre projet, créez deux clients OAuth :

- un client **Application Web** : son ID va dans `web_client_id`, et votre serveur vérifie que les jetons d’ID lui sont bien destinés ;
- un client **Android** avec votre nom de package et l’empreinte SHA-1 de la clé qui signe l’APK (`keytool -list -v -keystore <your.keystore>`), un par clé de signature (debug et release). Sans lui, la connexion échoue avec `no_credential` ou `unknown`.

## Utilisation

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

| Méthode | Affiche |
| --- | --- |
| `sign_in(nonce)` | les comptes qui se sont déjà connectés, ou tous les comptes si aucun ne l’a fait |
| `sign_in_with_account_chooser(nonce)` | tous les comptes Google de l’appareil |
| `sign_in_with_button(nonce)` | le parcours prévu pour un bouton « Se connecter avec Google » |
| `sign_in_silently(nonce)` | rien si un compte s’est déjà connecté ; sinon `no_credential` |
| `cancel()` | interrompt la connexion en cours (`cancelled`) |
| `sign_out()` | oublie le compte : `signed_out` ou `sign_out_failed` |

Si le `nonce` est vide, il est généré automatiquement. `signed_in` fournit `id_token`, `email`, `display_name`, `given_name`, `family_name`, `picture`, `phone_number` et le `nonce` envoyé avec la requête. Renseignez `hosted_domain` pour ne proposer que les comptes d’un domaine Google Workspace, et `request_verified_phone_number` pour demander un numéro de téléphone vérifié.

### Codes d’erreur

| Code | Signification |
| --- | --- |
| `cancelled` | le joueur a fermé le panneau, ou le jeu a appelé `cancel()` |
| `no_credential` | aucun compte Google de cet appareil ne correspond à la requête |
| `interrupted` | la requête a été interrompue ; elle peut être relancée |
| `provider_configuration` | aucun fournisseur d’identifiants n’est configuré pour les comptes Google |
| `unsupported` | l’appareil ne prend pas en charge l’API Credential Manager |
| `unknown` | le fournisseur a échoué sans en donner la raison |
| `custom` | le fournisseur a renvoyé une erreur d’un type qui lui est propre |
| `unexpected_credential` | la réponse n’est pas un jeton d’ID Google |
| `invalid_token` | le jeton d’ID Google est illisible |
| `invalid_config` | `web_client_id` (ou un autre paramètre) est inutilisable |
| `busy` | une connexion ou une déconnexion est déjà en cours |
| `unsupported_platform` | la plateforme n’est pas Android, ou le runtime Android de Godot est absent |
| `bridge` | un appel à l’API Android a échoué |

### Côté serveur

Le jeton d’ID ne prouve rien tant que votre serveur ne l’a pas vérifié : la signature (clés publiques de Google), `aud` = l’ID de votre client Web, `iss` = `accounts.google.com` ou `https://accounts.google.com`, `exp` dans le futur, et `nonce` = celui que le jeu a envoyé. Voir [Vérifier le jeton d’ID Google côté serveur](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token).

## Compilation

`zsh tools/build.zsh linux|android [debug|release]` compile la bibliothèque dans `addons/hqf_google_signin/bin/` (Android : avec le NDK du SDK Android).

## Qualité

La CI du badge ci-dessus vérifie le projet une fois par jour (jamais à chaque push) : `cargo fmt`, `clippy` avec chaque avertissement traité comme une erreur (`all`, `pedantic`, `nursery`), tests unitaires, de propriétés (`proptest`) et de documentation, tests de mutation avec `cargo-mutants`, et `cargo-deny` pour les licences et les avis de sécurité. Le lecteur de jetons d’ID passe au fuzzing avec `cargo-fuzz` ; son corpus se trouve dans `fuzz/corpus/`.

## Licence

MIT, voir [LICENSE](LICENSE).
