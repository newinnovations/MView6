#!/bin/bash

cd /opt/mview6

dpkg --add-architecture arm64
apt-get update
apt-get --assume-yes install libgtk-4-dev:arm64 libdav1d-dev:arm64 gcc-aarch64-linux-gnu g++-aarch64-linux-gnu libclang-dev
rustup target add aarch64-unknown-linux-gnu

cargo build --target aarch64-unknown-linux-gnu --release # --features mupdf
