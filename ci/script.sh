#!/bin/bash

set -xeo pipefail

cargo test --no-default-features
cargo test $FEATURES

# cargo test for wasm and cargo check for android and osx targets doesn't
# need to be repeated on other os and arch.
if [ "$TRAVIS_OS_NAME" = "linux" ] && [ "$TRAVIS_CPU_ARCH" = "amd64" ]
then
    cargo test --no-default-features --target=i686-unknown-linux-gnu
    cargo test $FEATURES --target=i686-unknown-linux-gnu

    cargo wasi test --no-default-features
    cargo wasi test $FEATURES

    cargo check --target aarch64-linux-android
    cargo check --target armv7-linux-androideabi

    cargo check --target x86_64-apple-darwin
    cargo check --target aarch64-apple-darwin
fi
