# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

🇬🇧 [English](README.md) · 🇫🇷 [Français](README.fr.md) · 🇨🇳 [简体中文](README.zh-CN.md) · 🇮🇳 [हिन्दी](README.hi.md) · 🇪🇸 [Español](README.es.md) · 🇸🇦 [العربية](README.ar.md) · 🇧🇩 [বাংলা](README.bn.md) · 🇧🇷 [Português](README.pt-BR.md) · 🇮🇩 [Bahasa Indonesia](README.id.md) · 🇵🇰 [اردو](README.ur.md)

面向 Godot 4.7+ Android 游戏的 Google 登录（Google Sign-In）插件，用 Rust 编写，是一个基于
[godot-rust](https://godot-rust.github.io/) 的 GDExtension。

它通过 Godot 内置的 `JavaClassWrapper` 类和 `AndroidRuntime` 插件调用 Android
[Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg)
API：无需 Kotlin，也无需构建 Java 插件。

## 安装

1. 将 `addons/hqf_google_signin/` 复制到你的项目中。
2. 在 *Project > Project Settings > Plugins* 中启用 **HQF Google Sign-In**：它会把 Google 库加入 Android 导出包。
3. 在 Android 导出预设中勾选 *Gradle Build > Use Gradle Build*（Google 库是 Maven 依赖，只有 Gradle 构建才会把它们打包进去）。

预编译库（release 构建，debug 导出也使用它们）：Linux x86_64（编辑器）以及 Android arm64、arm32、x86_64。其他平台的编辑器同样可以导出，但导出插件需要编辑器已加载该扩展，才能列出 Google 库；这些编辑器请从源码构建（`cargo build`）。

## Google Cloud 配置

在你项目的 [Google Cloud 控制台](https://console.cloud.google.com/apis/credentials)中，创建两个 OAuth 客户端：

- 一个 **Web application** 类型的客户端：它的 ID 填入 `web_client_id`，你的服务器会校验 ID token 的接收方是否为该客户端；
- 一个 **Android** 客户端，填写你的包名和用于签名 APK 的密钥的 SHA-1（`keytool -list -v -keystore <your.keystore>`），每个签名密钥各建一个（debug 和 release）。缺少这个客户端时，登录会失败，错误码为 `no_credential` 或 `unknown`。

## 用法

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

| 方法 | 显示内容 |
| --- | --- |
| `sign_in(nonce)` | 之前登录过的账号；如果没有，则显示所有账号 |
| `sign_in_with_account_chooser(nonce)` | 设备上的所有 Google 账号 |
| `sign_in_with_button(nonce)` | 专为“Sign in with Google”按钮设计的流程 |
| `sign_in_silently(nonce)` | 如果有账号之前登录过，则不显示任何内容；否则返回 `no_credential` |
| `cancel()` | 停止正在进行的登录（`cancelled`） |
| `sign_out()` | 忘记该账号：返回 `signed_out` 或 `sign_out_failed` |

`nonce` 为空时会自动生成。`signed_in` 会返回 `id_token`、`email`、`display_name`、`given_name`、`family_name`、`picture`、`phone_number`，以及请求中带上的 `nonce`。设置 `hosted_domain` 可以只提供某个 Google Workspace 域内的账号；设置 `request_verified_phone_number` 可以请求一个已验证的手机号码。

### 错误码

| 错误码 | 含义 |
| --- | --- |
| `cancelled` | 玩家关闭了登录面板，或者游戏调用了 `cancel()` |
| `no_credential` | 此设备上没有与请求匹配的 Google 账号 |
| `interrupted` | 请求被中断，可以重试 |
| `provider_configuration` | 没有为 Google 账号配置凭据提供方 |
| `unsupported` | 设备不支持 Credential Manager API |
| `unknown` | 提供方失败，但没有说明原因 |
| `custom` | 提供方抛出了自定义类型的错误 |
| `unexpected_credential` | 返回的结果不是 Google ID token |
| `invalid_token` | 无法读取 Google ID token |
| `invalid_config` | `web_client_id`（或其他设置）不可用 |
| `busy` | 已有登录或退出登录操作正在进行 |
| `unsupported_platform` | 当前平台不是 Android，或者缺少 Godot Android 运行时 |
| `bridge` | 对 Android API 的调用失败 |

### 服务器端

ID token 只有经过你的服务器验证后才有意义。需要验证：签名（使用 Google 的公钥）、`aud` = 你的 Web 客户端 ID、`iss` = `accounts.google.com` 或 `https://accounts.google.com`、`exp` 尚未过期，以及 `nonce` = 游戏发送的那个值。参见[在服务器端验证 Google ID token](https://developers.google.com/identity/gsi/web/guides/verify-google-id-token)。

## 构建

`zsh tools/build.zsh linux|android [debug|release]` 会把库构建到
`addons/hqf_google_signin/bin/`（Android：使用 Android SDK 中的 NDK）。

## 质量

上方徽章对应的 CI 每天检查一次（不会在每次 push 时运行）：`cargo fmt`；`clippy`，所有警告都视为错误（`all`、`pedantic`、`nursery`）；单元测试、属性测试（`proptest`）和文档测试；用 `cargo-mutants` 做变异测试；以及用 `cargo-deny` 检查许可证和安全公告。ID token 读取器用 `cargo-fuzz` 做模糊测试，其语料库位于 `fuzz/corpus/`。

## 许可证

MIT，参见 [LICENSE](LICENSE)。
