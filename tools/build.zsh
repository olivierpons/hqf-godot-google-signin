#!/bin/zsh
# Builds the extension library and copies it into addons/hqf_google_signin/bin/.
# Usage: zsh tools/build.zsh linux|android [debug|release]
# android builds arm64, arm32 and x86_64 with the NDK of the Android SDK
# (ANDROID_HOME, default ~/android_sdk/sdk_root; NDK version in NDK_VERSION).
renice -n 19 -p $$ > /dev/null; ionice -c 3 -p $$
set -e
cd ${0:A:h}/..
platform=$1
profile=${2:-debug}
[[ $platform == (linux|android) ]] || { print -u2 "usage: build.zsh linux|android [debug|release]"; exit 2 }
[[ $profile == (debug|release) ]] || { print -u2 "profile must be debug or release"; exit 2 }
bin=addons/hqf_google_signin/bin
flags=()
[[ $profile == release ]] && flags=(--release)
mkdir -p $bin
# put_library SRC DEST: copies through a temporary file then renames it, so a Godot
# process that has the old library mapped keeps reading it unharmed.
put_library() { /bin/cp -f $1 $2.tmp && /bin/mv -f $2.tmp $2 }
case $platform in
	linux)
		cargo build $flags
		put_library target/$profile/libhqf_godot_google_signin.so \
			$bin/libhqf_google_signin.linux.$profile.x86_64.so
		;;
	android)
		ndk=${ANDROID_HOME:-$HOME/android_sdk/sdk_root}/ndk/${NDK_VERSION:-28.1.13356709}
		llvm=$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin
		[[ -d $llvm ]] || { print -u2 "NDK not found: $llvm"; exit 1 }
		export CLANG_PATH=$llvm/clang
		export CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=$llvm/aarch64-linux-android24-clang
		export CARGO_TARGET_ARMV7_LINUX_ANDROIDEABI_LINKER=$llvm/armv7a-linux-androideabi24-clang
		export CARGO_TARGET_X86_64_LINUX_ANDROID_LINKER=$llvm/x86_64-linux-android24-clang
		for target arch in aarch64-linux-android arm64 armv7-linux-androideabi arm32 \
			x86_64-linux-android x86_64; do
			rustup target add $target > /dev/null
			cargo build $flags --target $target
			put_library target/$target/$profile/libhqf_godot_google_signin.so \
				$bin/libhqf_google_signin.android.$profile.$arch.so
		done
		;;
esac
ls -la $bin
