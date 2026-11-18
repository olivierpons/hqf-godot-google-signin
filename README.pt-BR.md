# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

Google Sign-In para jogos Android feitos em Godot 4.7+, escrito em Rust como uma GDExtension [godot-rust](https://godot-rust.github.io/).

Ele controla a API [Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg) do Android por meio do `JavaClassWrapper` e do plugin `AndroidRuntime` nativos do Godot: sem Kotlin e sem plugin Java para compilar.

## Instalação

1. Copie `addons/hqf_google_signin/` para o seu projeto.
2. Ative **HQF Google Sign-In** em *Project > Project Settings > Plugins*: o plugin adiciona as bibliotecas do Google às exportações para Android.
3. No preset de exportação do Android, marque *Gradle Build > Use Gradle Build* (as bibliotecas do Google são dependências Maven, e só o build Gradle as inclui).

Bibliotecas pré-compiladas (builds de release, usadas também nas exportações de debug): Linux x86_64 (editor) e Android arm64, arm32, x86_64. Editores de outras plataformas também conseguem exportar, mas o plugin de exportação precisa da extensão carregada no editor para listar as bibliotecas do Google; nessas plataformas, compile a partir do código-fonte (`cargo build`).

## Configuração do Google Cloud

No [console do Google Cloud](https://console.cloud.google.com/apis/credentials) do seu projeto, crie dois clientes OAuth:

- um cliente **Aplicativo da Web**: o ID dele vai em `web_client_id`, e o seu servidor confere se os tokens de ID são destinados a ele;
- um cliente **Android** com o nome do seu pacote e a impressão digital SHA-1 da chave que assina o APK (`keytool -list -v -keystore <your.keystore>`), um para cada chave de assinatura (debug e release). Sem ele, o login falha com `no_credential` ou `unknown`.

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

| Método | Mostra |
| --- | --- |
| `sign_in(nonce)` | as contas que já fizeram login antes ou, se nenhuma fez, todas as contas |
| `sign_in_with_account_chooser(nonce)` | todas as contas do Google do dispositivo |
| `sign_in_with_button(nonce)` | o fluxo pensado para um botão “Fazer login com o Google” |
| `sign_in_silently(nonce)` | nada se uma conta já fez login antes; caso contrário, `no_credential` |
| `cancel()` | interrompe o login em andamento (`cancelled`) |
| `sign_out()` | esquece a conta: `signed_out` ou `sign_out_failed` |

Se o `nonce` estiver vazio, ele é gerado automaticamente. `signed_in` retorna `id_token`, `email`, `display_name`, `given_name`, `family_name`, `picture`, `phone_number` e o `nonce` enviado na solicitação. Defina `hosted_domain` para oferecer apenas contas de um domínio do Google Workspace, e `request_verified_phone_number` para pedir um número de telefone verificado.

### Códigos de erro

| Código | Significado |
| --- | --- |
| `cancelled` | o jogador fechou o painel, ou o jogo chamou `cancel()` |
| `no_credential` | nenhuma conta do Google deste dispositivo corresponde à solicitação |
| `interrupted` | a solicitação foi interrompida; dá para tentar de novo |
| `provider_configuration` | não há provedor de credenciais configurado para contas do Google |
| `unsupported` | o dispositivo não oferece suporte à API Credential Manager |
| `unknown` | o provedor falhou sem informar o motivo |
| `custom` | o provedor gerou um erro de um tipo próprio |
| `unexpected_credential` | a resposta não é um token de ID do Google |
| `invalid_token` | não foi possível ler o token de ID do Google |
| `invalid_config` | `web_client_id` (ou outra configuração) não pode ser usado |
| `busy` | já há um login ou logout em andamento |
| `unsupported_platform` | a plataforma não é Android, ou o runtime Android do Godot não está presente |
| `bridge` | uma chamada à API do Android falhou |

### No servidor

O token de ID não prova nada até o seu servidor verificá-lo: a assinatura (chaves públicas do Google), `aud` = o ID do seu cliente da Web, `iss` = `accounts.google.com` ou `https://accounts.google.com`, `exp` no futuro e `nonce` = o que o jogo enviou. Veja [Verificar o token de ID do Google no servidor](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token).

## Compilação

`zsh tools/build.zsh linux|android [debug|release]` compila a biblioteca em `addons/hqf_google_signin/bin/` (Android: com o NDK do SDK do Android).

## Qualidade

O CI do badge acima verifica o projeto uma vez por dia (nunca a cada push): `cargo fmt`, `clippy` com todos os avisos tratados como erros (`all`, `pedantic`, `nursery`), testes unitários, de propriedades (`proptest`) e de documentação, testes de mutação com `cargo-mutants`, e `cargo-deny` para licenças e alertas de segurança. O leitor de tokens de ID passa por fuzzing com `cargo-fuzz`; o corpus dele fica em `fuzz/corpus/`.

## Licença

MIT, veja [LICENSE](LICENSE).
