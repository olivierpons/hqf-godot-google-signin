#!/bin/zsh
# Runs the quality gate: format check, clippy with warnings as errors, tests, doctests.
# Usage: zsh tools/check.zsh
# GDRUST_GODOT_BIN defaults to the newest ~/godot/Godot_v4*-stable_linux.x86_64.
renice -n 19 -p $$ > /dev/null; ionice -c 3 -p $$
set -e
cd ${0:A:h}/..
if [[ -z $GDRUST_GODOT_BIN ]]; then
	export GDRUST_GODOT_BIN=$(ls ~/godot/Godot_v4*-stable_linux.x86_64 | sort -V | tail -1)
fi
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo nextest run --all-features
cargo test --doc --all-features
