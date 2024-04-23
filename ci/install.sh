#!/bin/bash

set -xeo pipefail

# cargo test for wasm and cargo check for android and osx targets doesn't
# need to be repeated on other os and arch.
if [ "$TRAVIS_OS_NAME" = "linux" ] && [ "$TRAVIS_CPU_ARCH" = "amd64" ]
then
    rustup target add wasm32-wasi
    cargo install cargo-wasi
    curl -L https://github.com/CraneStation/wasmtime/releases/download/dev/wasmtime-dev-x86_64-linux.tar.xz \
        | sudo tar xJf - --strip-components=1 -C /usr/local/bin wasmtime-dev-x86_64-linux/wasmtime

    rustup target add aarch64-linux-android
    rustup target add armv7-linux-androideabi

    rustup target add x86_64-apple-darwin
    rustup target add aarch64-apple-darwin
fi
