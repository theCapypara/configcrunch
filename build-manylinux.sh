#!/bin/bash
set -ex

mkdir -p /build-rust
cd /build-rust
curl -O https://static.rust-lang.org/dist/rust-1.69.0-x86_64-unknown-linux-gnu.tar.gz
tar -xf rust-1.69.0-x86_64-unknown-linux-gnu.tar.gz
cd rust-1.69.0-x86_64-unknown-linux-gnu
./install.sh

source $HOME/.cargo/env
