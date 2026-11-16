# hqf-godot-google-signin

[![CI](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml/badge.svg)](https://github.com/olivierpons/hqf-godot-google-signin/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)
[![Godot 4.7+](https://img.shields.io/badge/Godot-4.7%2B-478cbf.svg?logo=godotengine&logoColor=white)](https://godotengine.org/)
[![Rust 2024](https://img.shields.io/badge/Rust-edition%202024-orange.svg?logo=rust)](https://www.rust-lang.org/)
[![Mutation tested](https://img.shields.io/badge/mutation%20tested-cargo--mutants-blueviolet.svg)](https://mutants.rs/)

Google Sign-In for Godot 4.7+ Android games, written in Rust as a
[godot-rust](https://godot-rust.github.io/) GDExtension.

It drives the Android
[Credential Manager](https://developer.android.com/identity/sign-in/credential-manager-siwg)
API through Godot's built-in `JavaClassWrapper` and `AndroidRuntime` plugin: no Kotlin,
no Java plugin to build.

Status: work in progress.

## Quality

Checked once a day by the CI badge above (never on each push): `cargo fmt`, `clippy`
with every warning as an error (`all`, `pedantic`, `nursery`), unit, property
(`proptest`) and documentation tests, mutation testing with `cargo-mutants`, and
`cargo-deny` for licenses and security advisories. The ID token reader is fuzzed with
`cargo-fuzz`; its corpus lives in `fuzz/corpus/`.

## License

MIT, see [LICENSE](LICENSE).
